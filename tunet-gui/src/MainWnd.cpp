#include <MainWnd.hpp>
#include <iostream>

MainWnd::MainWnd() : m_model(), m_label(this)
{
    m_label.setText("Hello world!");
    m_model.queue_state(State::Net);
    QObject::connect(&m_model, &Model::flux_changed, this, &MainWnd::update_flux);
    m_model.queue(Action::Flux);
}

MainWnd::~MainWnd() {}

void MainWnd::update_flux()
{
    auto flux = m_model.flux();
    m_label.setText(QString::fromUtf8(flux.username.data(), flux.username.size()));
}
