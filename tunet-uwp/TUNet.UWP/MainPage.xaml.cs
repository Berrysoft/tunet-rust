using TUNet.Interop;
using Windows.UI.Xaml;
using Windows.UI.Xaml.Controls;

namespace TUNet.UWP
{
    public sealed partial class MainPage : Page
    {
        public MainPage()
        {
            this.InitializeComponent();
        }

        public Model Model => ((App)Application.Current).Model;
    }
}
