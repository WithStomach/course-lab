int test(int a, int b)
{
    a = 0;
    a = a + c;
    return b;
}
int c = 0;
int main()
{
    int a = 0;
    a = a + 1;
    a = test(a, 1);
    return a;
}