#  简答作业

## 1.
**正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容 (运行 Rust 三个 bad 测例 (ch2b_bad_*.rs) ， 注意在编译时至少需要指定 LOG=ERROR 才能观察到内核的报错信息) ， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本**

sbi版本:RustSBI version 0.3.0-alpha.2, adapting to RISC-V SBI v1.0.0
- ch2b_bad_address: 该程序是对地址0写入0,访问了非法地址，因此出现了Page Fault错误，页表内无该数据  
[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003c4, kernel killed it.
- ch2b_bad_instructions: 该程序是执行了特权指令sret，因为用户程序处于U用户态，无法执行特权指令，因此内核会将其杀掉
  [kernel] IllegalInstruction in application, kernel killed it.
- ch2b_bad_register: 该程序访问了特权寄存器sstatus，同样是非法操作，内核会将其杀掉
  [kernel] IllegalInstruction in application, kernel killed it.

## 2.
**深入理解 trap.S 中两个函数 alltraps 和 restore 的作用，并回答如下问题:**

**L40：刚进入 restore 时，a0 代表了什么值。请指出 restore 的两种使用情景。**

**L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。**

1. a0代表了系统调用的返回值，__restore使用场景：系统调用的返回(从内核态返回用户态)和中断异常处理程序返回
2. 分别恢复sstatus,sepc,sscratch三个寄存器的值
   - sstatus: SPP字段记录Trap发生之前处于什么特权级(U/S)，sret指令执行时会根据该值来决定改变什么状态
   - sepc:记录了trap发生之前的指令地址，能够使得内核能够返回正确的用户地址空间
   - sscratch:将sscratch重新指向内核栈栈顶
3. x1寄存器是ra寄存器，是保存返回地址的，x3寄存器是Global Pointer,x2是sp(Stack Pointer)寄存器，接下来的代码会处理，x4是thread pointer寄存器，由于本章的内核都是单线程的，用不上。
4. sscratch指向 内核栈栈顶，sp指向用户栈栈顶
5. sret发生状态变换，该指令是特权指令，由于sstatus的SPP位是1，会将状态从S转换成U。
6. sp指向内核栈栈顶，sscratch指向用户栈栈顶
7. ecall



# 荣誉准测

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

   > 无

2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

   > The RISC-V Instruction Set Manual Volume II: Privileged Architecture
   >
   > # rCore-Tutorial-Book 第三版第二章[#](https://rcore-os.cn/rCore-Tutorial-Book-v3/index.html#rcore-tutorial-book)
   >
   > 

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

