int main()
{
    int a = 0;
    int b = 2, c = 3, d = 3, e = 4;

    if (b > c)
    {
        a = a + 0x1;
    }
    if (b < d)
    {
        a = a + 0x02;
    }

    return a;
}