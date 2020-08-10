source_filename = "hello.eln"

declare i32 @puts(i8*)

declare i32 @printf(i8*, ...)

define i64 @fun_stuff(i64 %0) {
entry:
  %n = alloca i64
  %tmpload = load i64, i64* %n
  ret void
}

define void @main() {
entry:
  %x = alloca i64
  %y = alloca float
  %tmpload = load i64, i64* %x
  %tmpcall = call i64 @fun_stuff(i64 %tmpload)
  %tmpload1 = load i64, i64* %x
  %tmpcmp = icmp eq i64 %tmpload1, float 0x4059066660000000
  br i1 %tmpcmp, label %lbl0, label %lbl1

lbl0:                                             ; preds = %entry
  %tmpload2 = load float, float* %y
  br label %lbl2

lbl1:                                             ; preds = %entry
  %tmpload3 = load i64, i64* %x
  br label %lbl2

lbl2:                                             ; preds = %lbl1, %lbl0
  ret void
}
