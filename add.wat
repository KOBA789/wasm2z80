(module
  (type $t0 (func (param i32 i32) (result i32)))
  (func $addTwo (export "addTwo") (type $t0) (param $p0 i32) (param $p1 i32) (result i32)
    (local.get $p0)
    (block $branch
      (br_if $branch
        (local.get $p0)
        (i32.const 0)
        (i32.eq))
    )
    (local.get $p1)
    (i32.add)))
