using TUNet.Interop;

namespace TUNet.UWP
{
    public static class Program
    {
        static int Main()
        {
            return Model.Start(m =>
            {
                Windows.UI.Xaml.Application.Start(_ => new App(m));
                return 0;
            });
        }
    }
}
