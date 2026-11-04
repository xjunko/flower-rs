#include <flower.h>

int main(void)
{
    static const char message[] = "hello from C\n";
    write(1, message, sizeof(message) - 1);
    return 0;
}
