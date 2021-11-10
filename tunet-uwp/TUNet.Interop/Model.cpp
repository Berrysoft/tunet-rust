#include "pch.h"

#include "Model.h"

#include "Model.g.cpp"

#include <chrono>
#include <cstddef>
#include <cstdint>
#include <functional>

using namespace winrt;
using namespace Windows::Foundation;
using namespace Windows::Foundation::Collections;
using namespace Windows::UI::Xaml::Data;
using namespace Windows::UI::Xaml::Input;

extern "C"
{
    using NativeModel = const void*;

    struct OnlineUser
    {
        std::uint32_t address;
        std::int64_t login_time;
        std::uint64_t flux;
        std::uint8_t mac_address[6];
        bool has_mac;
        bool is_local;
    };

    struct Detail
    {
        std::int64_t login_time;
        std::int64_t logout_time;
        std::uint64_t flux;
    };

    struct DetailGroup
    {
        std::int64_t logout_date;
        std::uint64_t flux;
    };

    struct DetailGroupByTime
    {
        std::uint32_t logout_start_time;
        std::uint64_t flux;
    };

    using MainCallback = int (*)(NativeModel, void*);
    using UpdateCallback = void (*)(TUNet::Interop::UpdateMsg, void*);
    using StringCallback = void (*)(const wchar_t*, void*);
    using OnlinesForeachCallback = bool (*)(const OnlineUser*, void*);
    using DetailsForeachCallback = bool (*)(const Detail*, void*);
    using DetailsGroupedForeachCallback = bool (*)(const DetailGroup*, void*);
    using DetailsGroupedByTimeForeachCallback = bool (*)(const DetailGroupByTime*, void*);

    std::int32_t tunet_model_start(std::size_t val, MainCallback main, void* data);
    void tunet_model_set_update_callback(NativeModel m, UpdateCallback update, void* data);
    void tunet_model_queue(NativeModel m, TUNet::Interop::Action a);
    bool tunet_model_queue_cred_load(NativeModel m);
    void tunet_model_queue_cred(NativeModel m, const wchar_t* u, const wchar_t* p);
    void tunet_model_queue_state(NativeModel m, TUNet::Interop::State s);
    void tunet_model_queue_connect(NativeModel m, std::uint32_t addr);
    void tunet_model_queue_drop(NativeModel m, std::uint32_t addr);
    void tunet_model_status(NativeModel m, StringCallback f, void* data);
    void tunet_model_cred_username(NativeModel m, StringCallback f, void* data);
    void tunet_model_cred_password(NativeModel m, StringCallback f, void* data);
    TUNet::Interop::State tunet_model_state(NativeModel m);
    void tunet_model_log(NativeModel m, StringCallback f, void* data);
    void tunet_model_flux_username(NativeModel m, StringCallback f, void* data);
    std::uint64_t tunet_model_flux_flux(NativeModel m);
    std::int64_t tunet_model_flux_online_time(NativeModel m);
    double tunet_model_flux_balance(NativeModel m);
    void tunet_model_onlines_foreach(NativeModel m, OnlinesForeachCallback f, void* data);
    void tunet_model_details_foreach(NativeModel m, DetailsForeachCallback f, void* data);
    bool tunet_model_log_busy(NativeModel m);
    bool tunet_model_online_busy(NativeModel m);
    bool tunet_model_detail_busy(NativeModel m);
    void tunet_model_set_del_at_exit(NativeModel m, bool v);
}

namespace winrt::TUNet::Interop::implementation
{
    struct Command : winrt::implements<Command, ICommand>
    {
        Command(std::function<void(IInspectable const&)> execute, std::function<bool(IInspectable const&)> can_execute)
            : m_execute(execute), m_can_execute(can_execute)
        {
        }

        void Execute(IInspectable const& parameter)
        {
            m_execute(parameter);
        }

        bool CanExecute(IInspectable const& parameter)
        {
            return m_can_execute(parameter);
        }

        event_token CanExecuteChanged(EventHandler<IInspectable> const& handler) { return m_can_execute_changed.add(handler); }
        void CanExecuteChanged(event_token const& token) { m_can_execute_changed.remove(token); }

        event<EventHandler<IInspectable>> m_can_execute_changed{};

        std::function<void(IInspectable const&)> m_execute{};
        std::function<bool(IInspectable const&)> m_can_execute{};
    };

    static void fn_string_callback(const wchar_t* data, void* d)
    {
        hstring* pstr = reinterpret_cast<hstring*>(d);
        *pstr = data;
    }

    template <typename F, typename... Args>
    hstring get_hstring(F&& f, Args... args)
    {
        hstring str{};
        f(std::move(args)..., fn_string_callback, &str);
        return str;
    }

    static void fn_update_callback(UpdateMsg m, void* data)
    {
        auto model = reinterpret_cast<Model*>(data);
        model->Update(m);
    }

    Model::Model(NativeModel handle) : m_handle(handle)
    {
        m_onlines = single_threaded_observable_vector<Online>();
        m_details = single_threaded_observable_vector<Detail>();
        m_info_command = winrt::make<Command>(
            [weak_self = get_weak()](IInspectable const&)
            {
                if (auto strong = weak_self.get())
                {
                    strong->Queue(Action::Flux);
                }
            },
            [weak_self = get_weak()](IInspectable const&)
            {
                return true;
            });

        tunet_model_set_update_callback(m_handle, fn_update_callback, this);
    }

    Model::~Model()
    {
        tunet_model_set_update_callback(m_handle, nullptr, nullptr);
    }

    event_token Model::PropertyChanged(PropertyChangedEventHandler const& handler)
    {
        return m_property_changed.add(handler);
    }

    void Model::PropertyChanged(event_token const& token) noexcept
    {
        m_property_changed.remove(token);
    }

    hstring Model::Status()
    {
        return get_hstring(tunet_model_status, m_handle);
    }

    Cred Model::Credential()
    {
        auto username = get_hstring(tunet_model_cred_username, m_handle);
        auto password = get_hstring(tunet_model_cred_password, m_handle);
        return { username, password };
    }

    void Model::Credential(TUNet::Interop::Cred const& cred)
    {
        tunet_model_queue_cred(m_handle, cred.Username.c_str(), cred.Password.c_str());
    }

    State Model::Method()
    {
        return tunet_model_state(m_handle);
    }

    void Model::Method(State state)
    {
        tunet_model_queue_state(m_handle, state);
    }

    hstring Model::Log()
    {
        return get_hstring(tunet_model_log, m_handle);
    }

    Info Model::OnlineInfo()
    {
        auto username = get_hstring(tunet_model_flux_username, m_handle);
        auto f = tunet_model_flux_flux(m_handle);
        auto online = tunet_model_flux_online_time(m_handle);
        auto balance = tunet_model_flux_balance(m_handle);
        return Info{ username, f, std::chrono::seconds{ online }, balance };
    }

    static int fn_init_callback(NativeModel handle, void* data)
    {
        TUNet::Interop::ModelStartHandler handler{};
        winrt::attach_abi(handler, data);
        auto model = winrt::make<Model>(handle);
        return handler(model);
    }

    std::int32_t Model::Start(ModelStartHandler const& handler)
    {
        return tunet_model_start(4, fn_init_callback, winrt::get_abi(handler));
    }

    void Model::Queue(Action action)
    {
        tunet_model_queue(m_handle, action);
    }

    static std::wstring_view get_string_repr(UpdateMsg msg)
    {
        switch (msg)
        {
        case UpdateMsg::Credential:
            return L"Credential";
        case UpdateMsg::Method:
            return L"Method";
        case UpdateMsg::Log:
            return L"Log";
        case UpdateMsg::Flux:
            return L"Flux";
        case UpdateMsg::Onlines:
            return L"Onlines";
        case UpdateMsg::Details:
            return L"Details";
        case UpdateMsg::LogBusy:
            return L"LogBusy";
        case UpdateMsg::OnlineBusy:
            return L"OnlineBusy";
        case UpdateMsg::DetailBusy:
            return L"DetailBusy";
        default:
            return {};
        }
    }

    void Model::Update(UpdateMsg msg)
    {
        switch (msg)
        {
        case UpdateMsg::Onlines:
            UpdateOnlines();
            break;
        case UpdateMsg::Details:
            UpdateDetails();
            break;
        }
        m_property_changed(*this, PropertyChangedEventArgs{ get_string_repr(msg) });
    }

    static constexpr MacAddress get_mac(std::uint8_t const (&maddr)[6]) noexcept
    {
        return { maddr[0], maddr[1], maddr[2], maddr[3], maddr[4], maddr[5] };
    }

    static bool fn_foreach_online(const OnlineUser* u, void* data)
    {
        auto& model = *reinterpret_cast<Model*>(data);
        model.m_onlines.Append(Online{
            u->address,
            clock::from_time_t(u->login_time),
            u->flux,
            u->has_mac ? std::make_optional(get_mac(u->mac_address)) : std::nullopt,
            u->is_local });
        return true;
    }

    void Model::UpdateOnlines()
    {
        tunet_model_onlines_foreach(m_handle, fn_foreach_online, this);
    }

    static bool fn_foreach_detail(const ::Detail* d, void* data)
    {
        auto& model = *reinterpret_cast<Model*>(data);
        model.m_details.Append(Detail{
            clock::from_time_t(d->login_time),
            clock::from_time_t(d->logout_time),
            d->flux });
        return true;
    }

    void Model::UpdateDetails()
    {
        tunet_model_details_foreach(m_handle, fn_foreach_detail, this);
    }
} // namespace winrt::TUNet::Interop::implementation
