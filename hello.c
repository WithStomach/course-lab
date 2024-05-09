int main()
{
    int a = 0;
    {
        int a = 3;
        a = a + 1;
        return a;
    }
    a = a + 1;
    return a;
}
