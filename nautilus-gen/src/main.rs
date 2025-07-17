use nautilus_gen::nautilus_gen;
fn main() {
    let naut = nautilus_gen!(

    A = B "+" C;
    B = 2.0;
    C = 400000L;
                    );

    println!("{:?}", naut)
}
