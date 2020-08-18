source_filename = "quicksort.eln"

declare i32 @puts(i8*)

define void @quicksort10([10 x i32] %0, i32 %1, i32 %2) {
entry:
  %a = alloca [10 x i32]
  store [10 x i32] %0, [10 x i32]* %a
  %start = alloca i32
  store i32 %1, i32* %start
  %len = alloca i32
  store i32 %2, i32* %len
  %tmpload = load i32, i32* %len
  %tmpcmp = icmp slt i32 %tmpload, 2
  br i1 %tmpcmp, label %lbl0, label %lbl1

lbl0:                                             ; preds = %entry
  ret void

lbl1:                                             ; preds = %entry
  br label %lbl2

lbl2:                                             ; preds = %lbl1
  %tmpload1 = load [10 x i32], [10 x i32]* %a
  %tmpload2 = load i32, i32* %start
  %tmpadd = add nsw i32 0, %tmpload2
  %tmpgep = getelementptr [10 x i32], [10 x i32]* %a, i32 0, i32 %tmpadd
  %tmpload3 = load i32, i32* %tmpgep
  %pivot = alloca i32
  store i32 %tmpload3, i32* %pivot
  %i = alloca i32
  store i32 0, i32* %i
  %j = alloca i32
  store i32 9, i32* %j
  br label %lbl3

lbl3:                                             ; preds = %lbl14, %lbl2
  br i1 true, label %lbl4, label %lbl5

lbl4:                                             ; preds = %lbl3
  br label %lbl6

lbl6:                                             ; preds = %lbl7, %lbl4
  %tmpload4 = load [10 x i32], [10 x i32]* %a
  %tmpload5 = load i32, i32* %start
  %tmpload6 = load i32, i32* %i
  %tmpadd7 = add nsw i32 %tmpload6, %tmpload5
  %tmpgep8 = getelementptr [10 x i32], [10 x i32]* %a, i32 0, i32 %tmpadd7
  %tmpload9 = load i32, i32* %tmpgep8
  %tmpload10 = load i32, i32* %pivot
  %tmpcmp11 = icmp slt i32 %tmpload9, %tmpload10
  br i1 %tmpcmp11, label %lbl7, label %lbl8

lbl7:                                             ; preds = %lbl6
  %tmpload12 = load i32, i32* %i
  %tmpadd13 = add nsw i32 1, %tmpload12
  store i32 %tmpadd13, i32* %i
  br label %lbl6

lbl8:                                             ; preds = %lbl6
  br label %lbl9

lbl9:                                             ; preds = %lbl10, %lbl8
  %tmpload14 = load [10 x i32], [10 x i32]* %a
  %tmpload15 = load i32, i32* %start
  %tmpload16 = load i32, i32* %j
  %tmpadd17 = add nsw i32 %tmpload16, %tmpload15
  %tmpgep18 = getelementptr [10 x i32], [10 x i32]* %a, i32 0, i32 %tmpadd17
  %tmpload19 = load i32, i32* %tmpgep18
  %tmpload20 = load i32, i32* %pivot
  %tmpcmp21 = icmp slt i32 %tmpload19, %tmpload20
  br i1 %tmpcmp21, label %lbl10, label %lbl11

lbl10:                                            ; preds = %lbl9
  %tmpload22 = load i32, i32* %j
  %tmpsub = sub nsw i32 %tmpload22, 1
  store i32 %tmpsub, i32* %j
  br label %lbl9

lbl11:                                            ; preds = %lbl12, %lbl9
  %tmpload23 = load i32, i32* %i
  %tmpload24 = load i32, i32* %j
  %tmpcmp25 = icmp sge i32 %tmpload23, %tmpload24
  br i1 %tmpcmp25, label %lbl12, label %lbl13

lbl12:                                            ; preds = %lbl11
  br label %lbl11
  br label %lbl14

lbl13:                                            ; preds = %lbl11
  br label %lbl14

lbl14:                                            ; preds = %lbl13, %lbl12
  %tmpload26 = load [10 x i32], [10 x i32]* %a
  %tmpload27 = load i32, i32* %start
  %tmpload28 = load i32, i32* %i
  %tmpadd29 = add nsw i32 %tmpload28, %tmpload27
  %tmpgep30 = getelementptr [10 x i32], [10 x i32]* %a, i32 0, i32 %tmpadd29
  %tmpload31 = load i32, i32* %tmpgep30
  %temp = alloca i32
  store i32 %tmpload31, i32* %temp
  %tmpload32 = load [10 x i32], [10 x i32]* %a
  %tmpload33 = load i32, i32* %start
  %tmpload34 = load i32, i32* %j
  %tmpadd35 = add nsw i32 %tmpload34, %tmpload33
  %tmpgep36 = getelementptr [10 x i32], [10 x i32]* %a, i32 0, i32 %tmpadd35
  %tmpload37 = load i32, i32* %tmpgep36
  %tmpload38 = load i32, i32* %start
  %tmpload39 = load i32, i32* %i
  %tmpadd40 = add nsw i32 %tmpload39, %tmpload38
  %tmpgep41 = getelementptr [10 x i32], [10 x i32]* %a, i32 0, i32 %tmpload37
  store i32 %tmpadd40, i32* %tmpgep41
  %tmpload42 = load i32, i32* %temp
  %tmpload43 = load i32, i32* %start
  %tmpload44 = load i32, i32* %j
  %tmpadd45 = add nsw i32 %tmpload44, %tmpload43
  %tmpgep46 = getelementptr [10 x i32], [10 x i32]* %a, i32 0, i32 %tmpload42
  store i32 %tmpadd45, i32* %tmpgep46
  br label %lbl3

lbl5:                                             ; preds = %lbl3
  %tmpload47 = load [10 x i32], [10 x i32]* %a
  %tmpload48 = load i32, i32* %start
  %tmpload49 = load i32, i32* %i
  call void @quicksort10([10 x i32] %tmpload47, i32 %tmpload48, i32 %tmpload49)
  %tmpload50 = load [10 x i32], [10 x i32]* %a
  %tmpload51 = load i32, i32* %start
  %tmpload52 = load i32, i32* %i
  %tmpadd53 = add nsw i32 %tmpload52, %tmpload51
  %tmpload54 = load i32, i32* %len
  %tmpload55 = load i32, i32* %i
  %tmpsub56 = sub nsw i32 %tmpload54, %tmpload55
  call void @quicksort10([10 x i32] %tmpload50, i32 %tmpadd53, i32 %tmpsub56)
  ret void
}

define void @main() {
entry:
  %arr = alloca [10 x i32]
  store [10 x i32] undef, [10 x i32]* %arr
  %tmpgep = getelementptr [10 x i32], [10 x i32]* %arr, i32 0, i32 1
  store i32 0, i32* %tmpgep
  %tmpgep1 = getelementptr [10 x i32], [10 x i32]* %arr, i32 0, i32 2
  store i32 1, i32* %tmpgep1
  %tmpgep2 = getelementptr [10 x i32], [10 x i32]* %arr, i32 0, i32 3
  store i32 2, i32* %tmpgep2
  %tmpgep3 = getelementptr [10 x i32], [10 x i32]* %arr, i32 0, i32 4
  store i32 3, i32* %tmpgep3
  %tmpgep4 = getelementptr [10 x i32], [10 x i32]* %arr, i32 0, i32 5
  store i32 4, i32* %tmpgep4
  %tmpgep5 = getelementptr [10 x i32], [10 x i32]* %arr, i32 0, i32 6
  store i32 5, i32* %tmpgep5
  %tmpgep6 = getelementptr [10 x i32], [10 x i32]* %arr, i32 0, i32 7
  store i32 6, i32* %tmpgep6
  %tmpgep7 = getelementptr [10 x i32], [10 x i32]* %arr, i32 0, i32 8
  store i32 7, i32* %tmpgep7
  %tmpgep8 = getelementptr [10 x i32], [10 x i32]* %arr, i32 0, i32 9
  store i32 8, i32* %tmpgep8
  %tmpgep9 = getelementptr [10 x i32], [10 x i32]* %arr, i32 0, i32 10
  store i32 9, i32* %tmpgep9
  %tmpload = load [10 x i32], [10 x i32]* %arr
  call void @quicksort10([10 x i32] %tmpload, i32 0, i32 10)
  ret void
}
