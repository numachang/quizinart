;(() => {
  // --- Custom confirm dialog (replaces browser confirm()) ---
  let pendingConfirm = null

  document.addEventListener('htmx:confirm', (evt) => {
    if (!evt.detail.question) return
    evt.preventDefault()

    const dialog = document.getElementById('confirm-dialog')
    if (!dialog) {
      evt.detail.issueRequest()
      return
    }

    dialog.querySelector('[data-confirm-message]').textContent =
      evt.detail.question
    pendingConfirm = () => evt.detail.issueRequest(true)
    dialog.showModal()
  })

  // --- Event delegation for clicks ---
  document.addEventListener('click', (e) => {
    // Confirm dialog OK
    if (e.target.closest('[data-confirm-ok]')) {
      const dialog = document.getElementById('confirm-dialog')
      if (dialog) dialog.close()
      if (pendingConfirm) {
        pendingConfirm()
        pendingConfirm = null
      }
      return
    }

    // Confirm dialog Cancel
    if (e.target.closest('[data-confirm-cancel]')) {
      const dialog = document.getElementById('confirm-dialog')
      if (dialog) dialog.close()
      pendingConfirm = null
      return
    }

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

  // --- Toast notifications ---
  const showToast = (message, type = 'error') => {
    let container = document.getElementById('toast-container')
    if (!container) {
      container = document.createElement('div')
      container.id = 'toast-container'
      container.setAttribute('aria-live', 'polite')
      document.body.appendChild(container)
    }

    const toast = document.createElement('div')
    toast.className = `toast toast-${type}`
    toast.setAttribute('role', 'alert')
    toast.textContent = message
    container.appendChild(toast)

    // Trigger enter animation
    requestAnimationFrame(() => toast.classList.add('toast-visible'))

    // Auto-dismiss after 5 seconds
    setTimeout(() => {
      toast.classList.remove('toast-visible')
      toast.addEventListener('transitionend', () => toast.remove())
    }, 5000)
  }

  // --- HTMX error handling ---
  let redirecting = false

  document.addEventListener('htmx:responseError', (evt) => {
    const status = evt.detail.xhr.status
    if (status === 401) {
      if (!redirecting) {
        redirecting = true
        window.location.href = '/login'
      }
      return
    }
    if (status === 403) {
      showToast('You do not have permission to perform this action.', 'error')
      return
    }
    if (status >= 500) {
      showToast('Something went wrong. Please try again.', 'error')
      return
    }
    showToast('Request failed. Please check your input.', 'error')
  })

  document.addEventListener('htmx:sendError', () => {
    showToast('Network error. Please check your connection.', 'error')
  })

  // --- Global loading bar ---
  document.addEventListener('htmx:beforeRequest', () => {
    const bar = document.getElementById('htmx-progress')
    if (bar) bar.classList.add('htmx-progress-active')
  })

  const hideProgress = () => {
    const bar = document.getElementById('htmx-progress')
    if (bar) bar.classList.remove('htmx-progress-active')
  }

  document.addEventListener('htmx:afterRequest', hideProgress)
  document.addEventListener('htmx:sendError', hideProgress)

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
