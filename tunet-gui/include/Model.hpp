#pragma once

#include <QObject>
#include <chrono>
#include <cstddef>
#include <cstdint>

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

bool tunet_start(std::size_t threads, int (*main)(int, char**), int argc, char** argv);

struct NetFlux
{
    std::string_view username;
    std::uint64_t flux;
    std::chrono::seconds online_time;
    double balance;
};

struct Model : QObject
{
    Q_OBJECT

public:
    Model();

    ~Model();

    NetFlux flux() const;

    void queue(Action a) const;
    void queue_state(State s) const;
    void update(UpdateMsg m) const;

signals:
    void flux_changed() const;

private:
    NativeModel m_handle{};
};
