int main()
{
    int e = 10;
    const int a = 11;
    e = e + 1;
    {
        int e = 1;
        e = e + a;
        const int a = 12;
        e = e + a;
        1 + 2;
    }
    return e + a;
}
