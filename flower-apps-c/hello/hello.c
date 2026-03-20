void _start() {
  int fd = 1;
  const char *msg = "hello world from userspace in c!\n";
  int len = 34;

  asm volatile("syscall" : : "a"(1), "D"(fd), "S"(msg), "d"(len));

  asm volatile("syscall" ::"a"(0));
}
