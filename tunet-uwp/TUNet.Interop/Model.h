#pragma once

#include "Model.g.h"

#include "winrt/TUNet.Interop.h"

namespace winrt::TUNet::Interop::implementation
{
    struct Model : ModelT<Model>
    {
        Model();
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
        Windows::Foundation::Collections::IObservableVector<TUNet::Interop::Online> Onlines();
        Windows::Foundation::Collections::IObservableVector<TUNet::Interop::Detail> Details();

        void Update(TUNet::Interop::UpdateMsg msg);

        void UpdateOnlines();
        void UpdateDetails();

        const void* m_handle{};
        event<Windows::UI::Xaml::Data::PropertyChangedEventHandler> m_propertyChangedEvent{};
        Windows::Foundation::Collections::IObservableVector<TUNet::Interop::Online> m_onlines{};
        Windows::Foundation::Collections::IObservableVector<TUNet::Interop::Detail> m_details{};
    };
} // namespace winrt::TUNet::Interop::implementation

namespace winrt::TUNet::Interop::factory_implementation
{
    struct Model : ModelT<Model, implementation::Model>
    {
    };
} // namespace winrt::TUNet::Interop::factory_implementation
