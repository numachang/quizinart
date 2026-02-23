;(() => {
  document.addEventListener('DOMContentLoaded', () => {
    const btn = document.getElementById('nav-toggle')
    const menu = document.getElementById('nav-menu')
    if (!btn || !menu) return

    btn.addEventListener('click', (e) => {
      e.stopPropagation()
      const open = menu.classList.toggle('open')
      btn.setAttribute('aria-expanded', open)
    })

    document.addEventListener('click', (e) => {
      if (!menu.classList.contains('open')) return
      if (!menu.contains(e.target) && e.target !== btn) {
        menu.classList.remove('open')
        btn.setAttribute('aria-expanded', 'false')
      }
    })

    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && menu.classList.contains('open')) {
        menu.classList.remove('open')
        btn.setAttribute('aria-expanded', 'false')
        btn.focus()
      }
    })
  })
})()
