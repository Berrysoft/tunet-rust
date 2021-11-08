#pragma once

#include <QColor>
#include <QDateTime>
#include <QObject>
#include <QString>
#include <algorithm>
#include <array>
#include <chrono>
#include <cstddef>
#include <cstdint>
#include <map>
#include <optional>
#include <vector>

namespace TUNet
{
    enum class Action : std::int32_t
    {
        Timer,
        Tick,
        Login,
        Logout,
        Flux,
        Online,
        Details,
    };

    enum class UpdateMsg : std::int32_t
    {
        Credential,
        State,
        Log,
        Flux,
        Online,
        Details,
        LogBusy,
        OnlineBusy,
        DetailBusy,
    };

    enum class State : std::int32_t
    {
        Auto,
        Net,
        Auth4,
        Auth6,
    };

    using NativeModel = const void*;

    struct Credential
    {
        QString username;
        QString password;
    };

    struct Flux
    {
        std::uint64_t m_value{};

        constexpr Flux() noexcept {}
        constexpr Flux(std::uint64_t value) noexcept : m_value(value) {}

        constexpr operator std::uint64_t() const { return m_value; }

        constexpr double toGb() const { return static_cast<double>(m_value) / 1000000000.0; }

        QString toString() const;
    };

    struct Ipv4Addr
    {
        std::uint32_t m_value{};

        constexpr Ipv4Addr() noexcept {}
        constexpr Ipv4Addr(std::uint32_t value) noexcept : m_value(value) {}
        explicit Ipv4Addr(const QString& str);

        constexpr operator std::uint32_t() const { return m_value; }

        QString toString() const;
    };

    struct MacAddress
    {
        std::array<std::uint8_t, 6> m_values{};

        constexpr MacAddress() noexcept {}
        explicit constexpr MacAddress(const std::uint8_t (&values)[6]) { std::copy(std::begin(values), std::end(values), m_values.begin()); }

        QString toString() const;
    };

    struct Info
    {
        QString username;
        Flux flux;
        std::chrono::seconds online_time;
        double balance;
    };

    struct Online
    {
        Ipv4Addr address;
        QDateTime login_time;
        Flux flux;
        std::optional<MacAddress> mac_address;
        bool is_local;
    };

    struct Detail
    {
        QDateTime login_time;
        QDateTime logout_time;
        Flux flux;
    };

    QString format_duration(std::chrono::seconds sec);
    QString format_datetime(const QDateTime& time);

    struct Model : QObject
    {
        Q_OBJECT

    public:
        using StartCallback = int (*)(int, char**, Model*);

        static std::int32_t start(std::size_t threads, StartCallback main, int argc, char** argv);

        Model(NativeModel handle);
        ~Model() override;

        QString status() const;
        Credential cred() const;
        State state() const;
        QColor accent_color() const;
        QString log() const;
        Info flux() const;
        std::vector<Online> onlines() const;
        std::vector<Detail> details() const;
        std::map<QDate, Flux> details_grouped() const;
        std::map<std::uint32_t, Flux> details_grouped_by_time(std::uint32_t groups) const;

        bool log_busy() const;
        bool online_busy() const;
        bool detail_busy() const;

        void queue(Action a) const;
        void queue_cred_load() const;
        void queue_cred(const Credential& cred) const;
        void queue_state(State s) const;
        void queue_connect(Ipv4Addr addr) const;
        void queue_drop(Ipv4Addr addr) const;
        void update(UpdateMsg m) const;

        void set_del_at_exit(bool v = true) const;

    signals:
        void ask_cred(const Credential& cred) const;

        void cred_changed() const;
        void state_changed() const;
        void log_changed() const;
        void flux_changed() const;
        void onlines_changed() const;
        void details_changed() const;

        void log_busy_changed() const;
        void online_busy_changed() const;
        void detail_busy_changed() const;

    private:
        NativeModel m_handle{};
    };

} // namespace TUNet
