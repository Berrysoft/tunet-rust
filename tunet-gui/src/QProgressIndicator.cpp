/*
 * The MIT License (MIT)
 *
 * Copyright (c) 2011 Morgan Leborgne
 * Copyright (c) 2021 Berrysoft
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

#include <QPainter>
#include <QProgressIndicator.hpp>
#include <QTimerEvent>

QProgressIndicator::QProgressIndicator(QWidget* parent)
    : QWidget(parent), m_angle(0), m_timerId(-1), m_delay(40), m_displayedWhenStopped(false), m_color(Qt::black)
{
    setFocusPolicy(Qt::NoFocus);
}

bool QProgressIndicator::isAnimated() const
{
    return m_timerId != -1;
}

void QProgressIndicator::setDisplayedWhenStopped(bool state)
{
    m_displayedWhenStopped = state;
    emit update();
}

bool QProgressIndicator::isDisplayedWhenStopped() const
{
    return m_displayedWhenStopped;
}

void QProgressIndicator::startAnimation()
{
    m_angle = 0;
    start();
}

void QProgressIndicator::stopAnimation()
{
    stop();
    emit update();
}

void QProgressIndicator::setAnimationDelay(int delay)
{
    stop();
    m_delay = delay;
    start();
}

void QProgressIndicator::setColor(const QColor& color)
{
    m_color = color;
    emit update();
}

int QProgressIndicator::heightForWidth(int w) const
{
    return w;
}

void QProgressIndicator::start()
{
    int timer;
    int old_id = m_timerId.load();
    while (old_id == -1)
    {
        timer = startTimer(m_delay);
        if (m_timerId.compare_exchange_weak(old_id, timer))
        {
            break;
        }
        else
        {
            killTimer(timer);
        }
    }
}

void QProgressIndicator::stop()
{
    int old_id = m_timerId.exchange(-1);
    if (old_id != -1)
        killTimer(old_id);
}

void QProgressIndicator::timerEvent(QTimerEvent* event)
{
    if (m_timerId == event->timerId())
    {
        m_angle = (m_angle + 30) % 360;
        emit update();
    }
}

void QProgressIndicator::paintEvent(QPaintEvent* event)
{
    if (m_displayedWhenStopped || isAnimated())
    {
        int width = qMin(this->width(), this->height());

        QPainter p(this);
        p.setRenderHint(QPainter::Antialiasing);
        p.setPen(Qt::NoPen);

        double outerRadius = (width - 1) * 0.5;
        double innerRadius = (width - 1) * 0.5 * 0.38;

        double capsuleHeight = outerRadius - innerRadius;
        double capsuleWidth = (width > 32) ? capsuleHeight * .23 : capsuleHeight * .35;
        double capsuleRadius = capsuleWidth / 2;

        for (int i = 0; i < 12; i++)
        {
            QColor color = m_color;
            color.setAlphaF(1.0f - (i / 12.0f));
            p.setBrush(color);
            p.save();
            p.translate(rect().center());
            p.rotate(m_angle - i * 30.0f);
            p.drawRoundedRect(-capsuleWidth * 0.5, -(innerRadius + capsuleHeight), capsuleWidth,
                              capsuleHeight, capsuleRadius, capsuleRadius);
            p.restore();
        }
    }
}
