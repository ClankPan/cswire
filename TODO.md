
- Fも係数として直接掛けれるようにしたい。現状はCoeffでラップしないといけない。
- utilsのインターフェースとして使う型がVなのは、しっくりこない。
- expr.rsのswitchboardのケースをテストしないとけない。oneの位置が正しいか？
- 

WireはWitnessとした割り当てられていて、VはWitnessの1次結合、という棲み分けがあるのだが、

メンタルモデルとして、Wireはwitnessそのものを表して、変数ではない。
VとVVが計算で扱える変数、なのだが、それだと毎回WireをVへ変換する作業が必要になるので、猥雑になってしまう。
直接計算に使えた方が読みやすい。

Wireを意識するのはどのタイミング？
InputとWitnessの割り当ての時だけ。
allocをinput,witnessの両方を用意して

VやVVをInputに割り当てる挙動があればいい？
let v = cs.wire(vv);
let v = cs.alloc(f);
let v = cs.input(f);
cs.anchor(vv,v);

vがwitness１つだけの場合は、(0,1,0)*(1,0,0) = a
cs.inptize(v) // vに含まれるのが、一つだけならそのwireをinputに変える、1次結合か2次結合なら、新しいwitnessとして割り当てる。

回路の外とのIOの抽象化をWireとする？

let v = hash(a);
let x = cs.wire(v * cs.one());
[0,1,0] * [1,0,0] = [0,1,0] のパターンになるけど、これは覗き穴最適化でなくすことができる。

cs.link((cs.one() - v) * v, cs.zero());

let wire = cs.wire(vv);
cs.link(vv, v);
cs.io(wire);
