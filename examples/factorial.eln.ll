source_filename = "factorial.eln"

declare i32 @puts(i8*)

define i32 @factorial(i32 %0) {
entry:
  %n = alloca i32
  store i32 %0, i32* %n
  %tmpload = load i32, i32* %n
  %tmpcmp = icmp eq i32 0, %tmpload
  br i1 %tmpcmp, label %lbl0, label %lbl1

lbl0:                                             ; preds = %entry
  ret i32 1

lbl1:                                             ; preds = %entry
  %tmpload1 = load i32, i32* %n
  %tmpload2 = load i32, i32* %n
  %tmpsub = sub i32 %tmpload2, 1
  %tmpcall = call i32 @factorial(i32 %tmpsub)
  %tmpmul = mul i32 %tmpcall, %tmpload1
  ret i32 %tmpmul
}

define i32 @main() {
entry:
  %tmpcall = call i32 @factorial(i32 5)
  %res = alloca i32
  store i32 %tmpcall, i32* %res
  %tmpload = load i32, i32* %res
  ret i32 %tmpload
}
