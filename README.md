For a more in depth explanation of what Nautilus grammar is look [here](https://react-h2020.eu/m/filer_public/cf/1e/cf1e4089-3103-4038-9b69-783d8a021a1a/ndss19-nautilus.pdf)
I'm trying to implement a macro that can create valid Nautilus syntax at compile time using the Rust macro system. This is intended to be compatible with LibAFL's [NautilusContext::with_rules](https://docs.rs/libafl/latest/libafl/generators/nautilus/struct.NautilusContext.html#method.with_rules) function

# Syntax By Example
## 3 Valid examples of a rule:
### Example 1
A = A + A;

A = 1;

Yields this vec:
```rust
[("A", "{A}+{A}"), ("A", "1")]
```

### Example 2
A = B  C;
B =1;
C=2;

Yields this vec:
```rust
[("A", "{B}{C}"), ("B", "1"), ("C", "2")]
```

### Example 3
A = B "+" C;
B = 2.0;
C = 400000L;

```rust
[("A", "{B}+{C}"), ("B", "2.0"), ("C", "400000L")]
```


# Syntax
Each rule has a name and an expression separated by an equals sign '=' and ends with a semicolon ";". A rule has one symbol on the left side of the equals sign, and a series of symbols on the right side. 

A symbol encapsulates the idea of non terminating and terminating expressions in the Nautilus paper linked above. A non terminating expression is any valid Rust identifier, just a rule name. A terminating expression is any Rust literal. As of right now, when a rule is made, there are no spaces between symbols even though the macro looks that way. You can also define an expression using literal syntax, which is just wrapping the expression in `""`.
# Bugs that need fixing

- [ ] Nonalphanumeric symbols register as a syntax error where a semicolon is expected, but still compiles normally
# Planned Features
- [ ] Usage of any length non alphanumeric sequences between rule names
- [ ] Defining separators at a grammar level and the ability to define separators at a rule level
- [ ] Include byte strings and C strings

