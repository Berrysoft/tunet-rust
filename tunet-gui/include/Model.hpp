#pragma once

#include <QColor>
#include <QDateTime>
#include <QObject>
#include <QString>
#include <array>
#include <chrono>
#include <cstddef>
#include <cstdint>
#include <map>
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
    };

    enum class State : std::int32_t
    {
        Auto,
        Net,
        Auth4,
        Auth6,
    };

    using NativeModel = const void*;

    QColor accent_color();

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

    struct Info
    {
        QString username;
        Flux flux;
        std::chrono::seconds online_time;
        double balance;
    };

    struct Online
    {
        std::uint32_t address;
        QDateTime login_time;
        Flux flux;
        std::array<std::uint8_t, 6> mac_address;
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
    QString format_ip(std::uint32_t addr);
    QString format_mac_address(const std::array<std::uint8_t, 6>& maddr);

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
        QString log() const;
        Info flux() const;
        std::vector<Online> onlines() const;
        std::vector<Detail> details() const;
        std::map<QDate, Flux> details_grouped() const;
        std::map<std::uint32_t, Flux> details_grouped_by_time(std::uint32_t groups) const;

        void queue(Action a) const;
        bool queue_cred_load() const;
        void queue_cred(const Credential& cred) const;
        void queue_state(State s) const;
        void update(UpdateMsg m) const;

    signals:
        void cred_changed() const;
        void state_changed() const;
        void log_changed() const;
        void flux_changed() const;
        void onlines_changed() const;
        void details_changed() const;

    private:
        NativeModel m_handle{};
    };

} // namespace TUNet
