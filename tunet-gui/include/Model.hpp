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

enum class StatusFlag : std::int32_t
{
    Unknown,
    Wwan,
    Wlan,
    Lan
};

struct Status
{
    StatusFlag flag;
    QString ssid;
};

enum class State : std::int32_t
{
    Auto,
    Net,
    Auth4,
    Auth6,
};

using NativeModel = const void*;

std::int32_t tunet_start(std::size_t threads, int (*main)(int, char**), int argc, char** argv);

QColor tunet_accent();

struct NetCredential
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

struct NetFlux
{
    QString username;
    Flux flux;
    std::chrono::seconds online_time;
    double balance;
};

struct NetUser
{
    std::uint32_t address;
    QDateTime login_time;
    Flux flux;
    std::array<std::uint8_t, 6> mac_address;
    bool is_local;
};

struct NetDetail
{
    QDateTime login_time;
    QDateTime logout_time;
    Flux flux;
};

QString tunet_format_status(const Status& status);
QString tunet_format_duration(std::chrono::seconds sec);
QString tunet_format_datetime(const QDateTime& time);
QString tunet_format_ip(std::uint32_t addr);
QString tunet_format_mac_address(const std::array<std::uint8_t, 6>& maddr);

struct Model : QObject
{
    Q_OBJECT

public:
    Model(QObject* parent = nullptr);
    ~Model() override;

    Status status() const;
    NetCredential cred() const;
    State state() const;
    QString log() const;
    NetFlux flux() const;
    std::vector<NetUser> onlines() const;
    std::vector<NetDetail> details() const;
    std::map<QDate, Flux> details_grouped() const;
    std::map<std::uint32_t, Flux> details_grouped_by_time(std::uint32_t groups) const;

    void queue(Action a) const;
    bool queue_read_cred() const;
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
