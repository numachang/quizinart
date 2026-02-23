;(() => {
  // --- Event delegation for clicks ---
  document.addEventListener('click', (e) => {
    // Dialog open: <element data-dialog-open="dialog-id">
    const dialogOpen = e.target.closest('[data-dialog-open]')
    if (dialogOpen) {
      e.preventDefault()
      const dialog = document.getElementById(
        dialogOpen.getAttribute('data-dialog-open'),
      )
      if (dialog) dialog.showModal()
      return
    }

    // Dialog close: <element data-dialog-close="dialog-id">
    const dialogClose = e.target.closest('[data-dialog-close]')
    if (dialogClose) {
      const dialog = document.getElementById(
        dialogClose.getAttribute('data-dialog-close'),
      )
      if (dialog) dialog.close()
      return
    }

    // Rename open: pre-fill rename dialog and open it
    // <element data-rename-name="..." data-rename-url="...">
    const renameOpen = e.target.closest('[data-rename-name]')
    if (renameOpen) {
      e.preventDefault()
      const nameInput = document.getElementById('rename-input')
      const urlInput = document.getElementById('rename-url')
      const dialog = document.getElementById('rename-dialog')
      if (nameInput)
        nameInput.value = renameOpen.getAttribute('data-rename-name')
      if (urlInput) urlInput.value = renameOpen.getAttribute('data-rename-url')
      if (dialog) dialog.showModal()
      return
    }

    // Rename submit: <element data-rename-submit>
    const renameSubmit = e.target.closest('[data-rename-submit]')
    if (renameSubmit) {
      const u = document.getElementById('rename-url')?.value
      const n = document.getElementById('rename-input')?.value
      if (u && n) {
        htmx.ajax('PATCH', u, {
          target: 'main',
          swap: 'innerHTML',
          values: { name: n },
        })
      }
      const dialog = document.getElementById('rename-dialog')
      if (dialog) dialog.close()
      return
    }

    // Copy share URL: <element data-copy-url>
    const copyUrl = e.target.closest('[data-copy-url]')
    if (copyUrl) {
      const urlInput = document.getElementById('share-url')
      if (urlInput) {
        navigator.clipboard.writeText(window.location.origin + urlInput.value)
      }
      return
    }

    // Select text on click: <input data-select-text>
    if (e.target.closest('[data-select-text]')?.select) {
      e.target.closest('[data-select-text]').select()
    }
  })

  // --- Enable submit button on form input change ---
  document.addEventListener('change', (e) => {
    if (e.target.closest('#question-form')) {
      const btn = document.getElementById('submit-btn')
      if (btn) btn.disabled = false
    }
  })

  // --- Session name auto-generation ---
  const generateSessionName = () => {
    const el = document.getElementById('session-name')
    if (el && !el.value) {
      const d = new Date()
      const y = d.getFullYear()
      const m = String(d.getMonth() + 1).padStart(2, '0')
      const dd = String(d.getDate()).padStart(2, '0')
      const r = Math.random().toString(36).substring(2, 8)
      el.value = `${y}-${m}-${dd}-${r}`
    }
  }

  // --- Dashboard charts ---
  let chartJsLoaded = typeof Chart !== 'undefined'

  const initCharts = () => {
    const el = document.getElementById('chart-data')
    if (!el) return
    const config = JSON.parse(el.getAttribute('data-config'))
    el.removeAttribute('id') // prevent re-initialization

    const createCharts = () => {
      const tc = getComputedStyle(document.documentElement).color

      const centerPlugin = (id, text, fontSize) => ({
        id,
        afterDraw: (c) => {
          const x = c.ctx
          x.save()
          x.fillStyle = tc
          x.font = `bold ${fontSize} sans-serif`
          x.textAlign = 'center'
          x.textBaseline = 'middle'
          const cx = (c.chartArea.left + c.chartArea.right) / 2
          const cy = (c.chartArea.top + c.chartArea.bottom) / 2
          x.fillText(text, cx, cy)
          x.restore()
        },
      })

      const doughnutOpts = {
        responsive: false,
        cutout: '70%',
        plugins: { legend: { display: false }, tooltip: { enabled: false } },
      }

      const ac = document.getElementById('answered-chart')
      if (ac) {
        new Chart(ac, {
          type: 'doughnut',
          data: {
            datasets: [
              {
                data: [config.uniqueAsked, config.remainingQuestions],
                backgroundColor: ['#4e79a7', '#e0e0e0'],
                borderWidth: 0,
              },
            ],
          },
          plugins: [centerPlugin('ct1', config.answeredCenter, '1.1rem')],
          options: doughnutOpts,
        })
      }

      const gc = document.getElementById('accuracy-chart')
      if (gc) {
        new Chart(gc, {
          type: 'doughnut',
          data: {
            datasets: [
              {
                data: [config.totalCorrect, config.totalIncorrect],
                backgroundColor: ['#59a14f', '#e0e0e0'],
                borderWidth: 0,
              },
            ],
          },
          plugins: [centerPlugin('ct2', config.accuracyCenter, '1.3rem')],
          options: doughnutOpts,
        })
      }

      const r = document.getElementById('radar-chart')
      if (r) {
        new Chart(r, {
          type: 'radar',
          data: {
            labels: config.radarLabels,
            datasets: [
              {
                data: config.radarData,
                backgroundColor: 'rgba(78,121,167,0.2)',
                borderColor: '#4e79a7',
                pointBackgroundColor: '#4e79a7',
                borderWidth: 2,
              },
            ],
          },
          options: {
            responsive: true,
            plugins: { legend: { display: false } },
            scales: { r: { min: 0, max: 100, ticks: { stepSize: 20 } } },
          },
        })
      }

      const dl = document.getElementById('daily-chart')
      if (dl) {
        new Chart(dl, {
          type: 'line',
          data: {
            labels: config.dailyLabels,
            datasets: [
              {
                data: config.dailyData,
                borderColor: '#4e79a7',
                backgroundColor: 'rgba(78,121,167,0.1)',
                fill: true,
                tension: 0.3,
                pointRadius: 4,
                pointHoverRadius: 6,
              },
            ],
          },
          options: {
            responsive: true,
            plugins: { legend: { display: false } },
            scales: {
              y: {
                min: 0,
                max: 100,
                title: { display: true, text: config.yLabel },
              },
              x: { title: { display: true, text: config.xLabel } },
            },
          },
        })
      }
    }

    if (chartJsLoaded) {
      createCharts()
    } else {
      const s = document.createElement('script')
      s.src = '/static/chart.min.js'
      s.onload = () => {
        chartJsLoaded = true
        createCharts()
      }
      document.head.appendChild(s)
    }
  }

  // Run on page load and HTMX content swaps
  const init = () => {
    generateSessionName()
    initCharts()
  }

  document.addEventListener('DOMContentLoaded', init)
  document.addEventListener('htmx:afterSettle', init)
})()
