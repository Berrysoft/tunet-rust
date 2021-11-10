#pragma once

#include "Model.g.h"

#include "winrt/TUNet.Interop.h"

namespace winrt::TUNet::Interop::implementation
{
    struct Model : ModelT<Model>
    {
        Model(const void* handle);
        ~Model();

        event_token PropertyChanged(Windows::UI::Xaml::Data::PropertyChangedEventHandler const& handler);
        void PropertyChanged(event_token const& token) noexcept;

        hstring Status();
        TUNet::Interop::Cred Credential();
        void Credential(TUNet::Interop::Cred const& cred);
        TUNet::Interop::State Method();
        void Method(TUNet::Interop::State state);
        hstring Log();
        TUNet::Interop::Info OnlineInfo();
        Windows::Foundation::Collections::IObservableVector<TUNet::Interop::Online> Onlines() { return m_onlines; }
        Windows::Foundation::Collections::IObservableVector<TUNet::Interop::Detail> Details() { return m_details; }

        Windows::UI::Xaml::Input::ICommand LoginCommand() { return m_login_command; }
        Windows::UI::Xaml::Input::ICommand LogoutCommand() { return m_logout_command; }
        Windows::UI::Xaml::Input::ICommand FetchInfoCommand() { return m_info_command; }
        Windows::UI::Xaml::Input::ICommand FetchOnlinesCommand() { return m_onlines_command; }
        Windows::UI::Xaml::Input::ICommand FetchDetailsCommand() { return m_details_command; }

        static std::int32_t Start(TUNet::Interop::ModelStartHandler const& handler);

        void Queue(TUNet::Interop::Action action);
        void Update(TUNet::Interop::UpdateMsg msg);

        void UpdateOnlines();
        void UpdateDetails();

        event<Windows::UI::Xaml::Data::PropertyChangedEventHandler> m_property_changed{};

        const void* m_handle{};
        Windows::Foundation::Collections::IObservableVector<TUNet::Interop::Online> m_onlines{};
        Windows::Foundation::Collections::IObservableVector<TUNet::Interop::Detail> m_details{};

        Windows::UI::Xaml::Input::ICommand m_login_command{};
        Windows::UI::Xaml::Input::ICommand m_logout_command{};
        Windows::UI::Xaml::Input::ICommand m_info_command{};
        Windows::UI::Xaml::Input::ICommand m_onlines_command{};
        Windows::UI::Xaml::Input::ICommand m_details_command{};
    };
} // namespace winrt::TUNet::Interop::implementation

namespace winrt::TUNet::Interop::factory_implementation
{
    struct Model : ModelT<Model, implementation::Model>
    {
    };
} // namespace winrt::TUNet::Interop::factory_implementation
