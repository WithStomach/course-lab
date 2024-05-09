int main()
{
    {
        int a = 1;
        a = 2;
    }
    int a = 0;
    {
        int a = 3;
    }
    a = a + 1;
    return a + 1;
}
