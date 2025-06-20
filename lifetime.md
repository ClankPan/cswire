

`wire`は`cs`とlifetimeを共有しているので、`cs`のスコープよりも長く生きることはできない。

```rust
let  wire;
{
    let cs = ConstraintSystemRef::<Fr>::new(); //  binding `cs` declared here [E0597]
    wire = cs.alloc(10); //`cs` does not live long enough borrowed value does not live long enough [E0597]
} // `cs` dropped here while still borrowed [E0597]
println!("{}", wire.raw()); // borrow later used here [E0597]
```

`ConstrainSystemRef`は`Rc<RefCell<ConstrainSystemRef>>>`のラッパーなので、所有権ごと関数に渡してしまうと、
Wireを生成したcsが関数内でドロップしてしまい、csよりも長く生きてしまうことになりエラーになる。
なので、`ConstrainSystemRef`と同じライフタイムであることを明示しなくてはいけない。


```rust
pub fn circuit<'a, F: Field>(cs: &'a ConstraintSystemRef<F>, value: F) -> Wire<'a, F> {
    cs.alloc(value)
}
```


これらを守るだけで、回路を、必要な値を受け取って必要なWitnessを計算して回路に制約を加える関数、として定義することが簡単になる。
