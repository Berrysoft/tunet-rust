#include <condition_variable>
#include <iostream>
#include <mutex>
#include <tunet.hpp>

void main_impl()
{
    std::mutex mtx{};
    std::condition_variable cd{};

    tunet::model model{
        [&cd](tunet::update_msg msg)
        {
            switch (msg)
            {
            case tunet::update_msg::flux:
                cd.notify_one();
                break;
            }
        }
    };

    model.state(tunet::state::net);

    {
        std::unique_lock lock{ mtx };
        model.queue(tunet::action::flux);
        cd.wait(lock);
        std::cout << model.flux_flux() / 1000000000.0 << std::endl;
    }
}

int main()
{
    tunet::init(4, main_impl);
}
