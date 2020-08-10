source_filename = "hello.eln"

declare i32 @puts(i8*)

declare i32 @printf(i8*, ...)

define void @main() {
entry:
  %x = alloca i64
  store i64 10, i64* %x
  %y = alloca float
  store float 0x40289999A0000000, float* %y
  %tmpload = load i64, i64* %x
  %tmpcmp = icmp slt i64 100, %tmpload
  br i1 %tmpcmp, label %lbl0, label %lbl1

lbl0:                                             ; preds = %entry
  %tmpload1 = load float, float* %y
  br label %lbl2

lbl1:                                             ; preds = %entry
  %tmpload2 = load i64, i64* %x
  br label %lbl2

lbl2:                                             ; preds = %lbl1, %lbl0
  ret void
}
