source_filename = "examples/test2.chi"

define void @f() {
entry:
  %x = alloca i32
  store i32 7, i32* %x
}

define void @g() {
entry:
  %tmpcall = call void @f()
}
