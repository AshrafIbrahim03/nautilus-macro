use libafl::generators::NautilusContext;
use nautilus_gen::nautilus_gen;

#[test]
fn is_accepted_by_nautilus_context() {
    let nautilus_grammar = nautilus_gen!(
            A = A + A;
            A = 1;
    );

    let ctx = NautilusContext::with_rules(10, nautilus_grammar.as_slice())
        .expect("Could not build simple NautilusContext from macro");
}
