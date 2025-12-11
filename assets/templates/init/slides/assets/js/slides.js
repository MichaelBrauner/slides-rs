/**
 * Slides.js - Presentation navigation
 * Keyboard (arrows, vim), touch swipe
 */

const current = parseInt(document.body.dataset.slide) || 1;
const total = parseInt(document.body.dataset.total) || 1;
let lastGPress = 0;

document.addEventListener('keydown', handleKeydown);
setupTouch();

function handleKeydown(e) {
  if (e.ctrlKey || e.altKey || e.metaKey) return;

  switch (e.key) {
    case 'ArrowRight': case 'PageDown': case ' ': case 'j': case 'l':
      e.preventDefault(); go(current + 1); break;
    case 'ArrowLeft': case 'PageUp': case 'k': case 'h':
      e.preventDefault(); go(current - 1); break;
    case 'Home':
      e.preventDefault(); go(1); break;
    case 'End': case 'G':
      e.preventDefault(); go(total); break;
    case 'g':
      e.preventDefault();
      if (Date.now() - lastGPress < 500) go(1);
      lastGPress = Date.now();
      break;
    case 'o': case 'O':
      e.preventDefault(); location.href = 'overview.html'; break;
    case 'p': case 'P':
      e.preventDefault(); openPresenter(); break;
    case 'Escape':
      e.preventDefault(); location.href = `slide-${current}.html`; break;
  }
}

function setupTouch() {
  let startX = 0;
  document.addEventListener('touchstart', e => startX = e.changedTouches[0].screenX);
  document.addEventListener('touchend', e => {
    const diff = startX - e.changedTouches[0].screenX;
    if (Math.abs(diff) > 50) go(diff > 0 ? current + 1 : current - 1);
  });
}

function go(n) {
  const target = Math.max(1, Math.min(total, n));
  if (target === current) return;
  location.href = `slide-${target}.html`;
}

function openPresenter() {
  window.open(
    `presenter/slide-${current}.html`,
    'presenter',
    `width=1200,height=800,left=${screenX + outerWidth},top=${screenY}`
  );
}

window.Slides = { go, current, total };
