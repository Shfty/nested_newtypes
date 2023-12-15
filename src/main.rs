/// Experiment to see if deref coercion will allow multiply-nested wrappers to expose
/// all related trait methods regardless of nesting order
use std::{any::TypeId, borrow::Cow, marker::PhantomData, ops::Deref};

// Peano numbers
struct Z;
struct S<T>(PhantomData<T>);

// Usage wrapper
// Tags a type as being related to some other type
enum UsageTag {}

struct Usage<U, T> {
    data: T,
    _phantom: PhantomData<U>,
}

impl<U, T> Usage<U, T> {
    pub fn new(data: T) -> Self {
        Usage {
            data,
            _phantom: Default::default(),
        }
    }
}

impl<U, T> Deref for Usage<U, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

// Usage method host to allow deref coercion for wrapping types
trait UsageTrait {
    type Usage;

    fn type_id(&self) -> TypeId
    where
        Self::Usage: 'static,
    {
        std::any::TypeId::of::<Self::Usage>()
    }
}

impl<U: 'static, T> UsageTrait for Usage<U, T> {
    type Usage = U;
}

// Changed wrapper
// Adds a boolean changed flag to some other type
struct ChangedWrap<T> {
    data: T,
    changed: bool,
}

impl<T> ChangedWrap<T> {
    pub fn new(data: T) -> Self {
        ChangedWrap {
            data,
            changed: false,
        }
    }
}

impl<T> Deref for ChangedWrap<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

struct Changed(pub bool);

// ChangedFlag method host to allow deref coercion for wrapping types
trait ChangedFlagTrait<T>: Sized {
    fn get_changed(&self) -> bool;
    fn set_changed(&mut self, changed: bool);
}

impl<T> ChangedFlagTrait<T> for ChangedWrap<T> {
    fn get_changed(&self) -> bool {
        self.changed
    }

    fn set_changed(&mut self, changed: bool) {
        self.changed = changed;
    }
}

// Label wrapper
// Adds a string label to some other type
struct LabelWrap<T> {
    data: T,
    label: Cow<'static, str>,
}

impl<T> LabelWrap<T> {
    pub fn new(data: T) -> Self {
        LabelWrap {
            data,
            label: "".into(),
        }
    }
}

impl<T> Deref for LabelWrap<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

struct Label<T>(T)
where
    T: Into<Cow<'static, str>>;

// Label method host to allow deref coercion for wrapping types
trait LabelTrait<T>: Sized {
    fn get_label(&self) -> &str;
    fn set_label<L>(&mut self, label: L)
    where
        L: Into<Cow<'static, str>>;
}

impl<T> LabelTrait<T> for LabelWrap<T> {
    fn get_label(&self) -> &str {
        &self.label
    }

    fn set_label<L>(&mut self, label: L)
    where
        L: Into<Cow<'static, str>>,
    {
        self.label = label.into();
    }
}

// Utility trait for constructing nested newtypes
trait Construct<T, I> {
    fn construct(t: T) -> Self;
}

/*
impl<V, T> Construct<V, Z> for T where T: InnerType<InnerType = V> {
    fn construct(t: T) -> Self {
        Usage::new(t)
    }
}
*/

impl<T, U> Construct<T, Z> for Usage<U, T> {
    fn construct(t: T) -> Self {
        Usage::new(t)
    }
}

impl<T, I, U, N> Construct<T, S<I>> for Usage<U, N>
where
    N: Construct<T, I>,
{
    fn construct(t: T) -> Self {
        Usage::new(N::construct(t))
    }
}

impl<T> Construct<T, Z> for LabelWrap<T> {
    fn construct(t: T) -> Self {
        LabelWrap::new(t)
    }
}

impl<T, I, N> Construct<T, S<I>> for LabelWrap<N>
where
    N: Construct<T, I>,
{
    fn construct(t: T) -> Self {
        LabelWrap::new(N::construct(t))
    }
}

impl<T> Construct<T, Z> for ChangedWrap<T> {
    fn construct(t: T) -> Self {
        ChangedWrap::new(t)
    }
}

impl<T, I, N> Construct<T, S<I>> for ChangedWrap<N>
where
    N: Construct<T, I>,
{
    fn construct(t: T) -> Self {
        ChangedWrap::new(N::construct(t))
    }
}

// Utility trait for treating a struct as its own builder
trait With<T, I> {
    fn with(self, t: T) -> Self;
}

// ChangedFlag impl
impl<T> With<Changed, Z> for ChangedWrap<T> {
    fn with(mut self, t: Changed) -> Self {
        self.set_changed(t.0);
        self
    }
}

impl<T, I, N> With<T, S<I>> for ChangedWrap<N>
where
    N: With<T, I>,
{
    fn with(mut self, t: T) -> Self {
        self.data = self.data.with(t);
        self
    }
}

// Label impl
impl<L, T> With<Label<L>, Z> for LabelWrap<T>
where
    L: Into<Cow<'static, str>>,
{
    fn with(mut self, t: Label<L>) -> Self {
        self.set_label(t.0.into());
        self
    }
}

impl<T, I, N> With<T, S<I>> for LabelWrap<N>
where
    N: With<T, I>,
{
    fn with(mut self, t: T) -> Self {
        self.data = self.data.with(t);
        self
    }
}

// Usage impl
impl<T, I, U, N> With<T, S<I>> for Usage<U, N>
where
    N: With<T, I>,
{
    fn with(mut self, t: T) -> Self {
        self.data = self.data.with(t);
        self
    }
}

// Entrypoint
fn main() {
    test_the_first();
    test_the_second();
}

// Let's see if this works...
fn test_the_first() {
    // Construct permutations of our wrapper types
    let usage_changed_label = Usage::<UsageTag, _>::new(ChangedWrap::new(LabelWrap::new(1234)))
        .with(Changed(true))
        .with(Label("one"));

    let usage_label_changed = Usage::<UsageTag, _>::new(LabelWrap::new(ChangedWrap::new(1234)))
        .with(Changed(true))
        .with(Label("two"));

    let changed_usage_label = ChangedWrap::new(Usage::<UsageTag, _>::new(LabelWrap::new(1234)))
        .with(Changed(false))
        .with(Label("three"));

    let changed_label_usage = ChangedWrap::new(LabelWrap::new(Usage::<UsageTag, _>::new(1234)))
        .with(Changed(false))
        .with(Label("four"));

    let label_usage_changed = LabelWrap::new(Usage::<UsageTag, _>::new(ChangedWrap::new(1234)))
        .with(Changed(true))
        .with(Label("five"));

    let label_changed_usage = LabelWrap::new(ChangedWrap::new(Usage::<UsageTag, _>::new(1234)))
        .with(Changed(false))
        .with(Label("six"));

    // Use the changed trait to retrieve changed flag from each permutation
    let changed = usage_changed_label.get_changed();
    println!("Changed: {}", changed);
    let changed = usage_label_changed.get_changed();
    println!("Changed: {}", changed);
    let changed = changed_usage_label.get_changed();
    println!("Changed: {}", changed);
    let changed = changed_label_usage.get_changed();
    println!("Changed: {}", changed);
    let changed = label_usage_changed.get_changed();
    println!("Changed: {}", changed);
    let changed = label_changed_usage.get_changed();
    println!("Changed: {}", changed);

    // Use the usage trait to retrieve usage type id from each permutation
    let type_id = usage_changed_label.type_id();
    println!("Type ID: {:?}", type_id);
    let type_id = usage_label_changed.type_id();
    println!("Type ID: {:?}", type_id);
    let type_id = changed_usage_label.type_id();
    println!("Type ID: {:?}", type_id);
    let type_id = changed_label_usage.type_id();
    println!("Type ID: {:?}", type_id);
    let type_id = label_usage_changed.type_id();
    println!("Type ID: {:?}", type_id);
    let type_id = label_changed_usage.type_id();
    println!("Type ID: {:?}", type_id);

    // Use the label trait to retrieve label string slice from each permutation
    let label = usage_changed_label.get_label();
    println!("Label: {}", label);
    let label = usage_label_changed.get_label();
    println!("Label: {}", label);
    let label = changed_usage_label.get_label();
    println!("Label: {}", label);
    let label = changed_label_usage.get_label();
    println!("Label: {}", label);
    let label = label_usage_changed.get_label();
    println!("Label: {}", label);
    let label = label_changed_usage.get_label();
    println!("Label: {}", label);

    // No compiler errors!
    //
    // All of this works because the compiler is able to traverse each permutation's deref chain transparently
    //
    // Practically, this means you can compose a structure of data from many discrete types
    // instead of needing to define a rigid struct that obscures its members behind an API,
    // or introduces the potential for breaking changes by making its members public.
    //
    // The set of member variables formed by these types remains freely accessible,
    // as the traits defining their functionality can be applied directly to any type
    // that dereferences into its implementor.
    //
    // That's rad.
}

fn test_the_second() {
    type One<T> = Usage<UsageTag, ChangedWrap<LabelWrap<T>>>;
    type Two<T> = Usage<UsageTag, LabelWrap<ChangedWrap<T>>>;
    type Three<T> = ChangedWrap<Usage<UsageTag, LabelWrap<T>>>;
    type Four<T> = ChangedWrap<LabelWrap<Usage<UsageTag, T>>>;
    type Five<T> = LabelWrap<Usage<UsageTag, ChangedWrap<T>>>;
    type Six<T> = LabelWrap<ChangedWrap<Usage<UsageTag, T>>>;

    // Construct permutations of our wrapper types
    let usage_changed_label = One::<u8>::construct(1)
        .with(Changed(true))
        .with(Label("one"));

    let usage_label_changed = Two::<u16>::construct(2)
        .with(Changed(true))
        .with(Label("two"));

    let changed_usage_label = Three::<u32>::construct(3)
        .with(Changed(false))
        .with(Label("three"));

    let changed_label_usage = Four::<u64>::construct(4)
        .with(Changed(false))
        .with(Label("four"));

    let label_usage_changed = Five::<u128>::construct(5)
        .with(Changed(true))
        .with(Label("five"));

    let label_changed_usage = Six::<usize>::construct(6)
        .with(Changed(false))
        .with(Label("six"));

    // Use the changed trait to retrieve changed flag from each permutation
    let changed = usage_changed_label.get_changed();
    println!("Changed: {}", changed);
    let changed = usage_label_changed.get_changed();
    println!("Changed: {}", changed);
    let changed = changed_usage_label.get_changed();
    println!("Changed: {}", changed);
    let changed = changed_label_usage.get_changed();
    println!("Changed: {}", changed);
    let changed = label_usage_changed.get_changed();
    println!("Changed: {}", changed);
    let changed = label_changed_usage.get_changed();
    println!("Changed: {}", changed);

    // Use the usage trait to retrieve usage type id from each permutation
    let type_id = usage_changed_label.type_id();
    println!("Type ID: {:?}", type_id);
    let type_id = usage_label_changed.type_id();
    println!("Type ID: {:?}", type_id);
    let type_id = changed_usage_label.type_id();
    println!("Type ID: {:?}", type_id);
    let type_id = changed_label_usage.type_id();
    println!("Type ID: {:?}", type_id);
    let type_id = label_usage_changed.type_id();
    println!("Type ID: {:?}", type_id);
    let type_id = label_changed_usage.type_id();
    println!("Type ID: {:?}", type_id);

    // Use the label trait to retrieve label string slice from each permutation
    let label = usage_changed_label.get_label();
    println!("Label: {}", label);
    let label = usage_label_changed.get_label();
    println!("Label: {}", label);
    let label = changed_usage_label.get_label();
    println!("Label: {}", label);
    let label = changed_label_usage.get_label();
    println!("Label: {}", label);
    let label = label_usage_changed.get_label();
    println!("Label: {}", label);
    let label = label_changed_usage.get_label();
    println!("Label: {}", label);
}
