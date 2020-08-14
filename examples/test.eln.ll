source_filename = "test.eln"

@tmpstr = private unnamed_addr constant [6 x i8] c"Woop!\00", align 1

declare i32 @puts(i8*)

define i32 @main() {
entry:
  %x = alloca i32
  store i32 0, i32* %x
  br label %lbl0

lbl0:                                             ; preds = %lbl1, %entry
  %tmpload = load i32, i32* %x
  %tmpcmp = icmp slt i32 %tmpload, 10
  br i1 %tmpcmp, label %lbl1, label %lbl2

lbl1:                                             ; preds = %lbl0
  %tmpcall = call i32 @puts(i8* getelementptr inbounds ([6 x i8], [6 x i8]* @tmpstr, i32 0, i32 0))
  %tmpload1 = load i32, i32* %x
  %tmpadd = add nsw i32 1, %tmpload1
  store i32 %tmpadd, i32* %x
  br label %lbl0

lbl2:                                             ; preds = %lbl0
  ret i32 0
}
