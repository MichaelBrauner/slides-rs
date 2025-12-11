/**
 * Presenter Mode - Speaker view with notes, timer, and keyboard navigation
 * Each slide now has its own presenter page (presenter/slide-N.html)
 */

const current = parseInt(document.body.dataset.current);
const total = parseInt(document.body.dataset.total);

// Persist timer across page navigations using sessionStorage
let startTime = sessionStorage.getItem('presenterStartTime');
if (!startTime) {
    startTime = Date.now();
    sessionStorage.setItem('presenterStartTime', startTime);
} else {
    startTime = parseInt(startTime);
}

// Timer update
setInterval(() => {
    const s = Math.floor((Date.now() - startTime) / 1000);
    document.getElementById('timer').textContent =
        `${String(Math.floor(s / 3600)).padStart(2, '0')}:${String(Math.floor((s % 3600) / 60)).padStart(2, '0')}:${String(s % 60).padStart(2, '0')}`;
}, 1000);

// Clock update
setInterval(() => {
    document.getElementById('clock').textContent = new Date().toLocaleTimeString();
}, 1000);
document.getElementById('clock').textContent = new Date().toLocaleTimeString();

// Keyboard navigation
document.addEventListener('keydown', e => {
    if (e.ctrlKey || e.altKey || e.metaKey) return;
    switch (e.key) {
        case 'ArrowRight':
        case 'PageDown':
        case ' ':
        case 'j':
        case 'l':
            if (current < total) {
                e.preventDefault();
                location.href = `slide-${current + 1}.html`;
            }
            break;
        case 'ArrowLeft':
        case 'PageUp':
        case 'k':
        case 'h':
            if (current > 1) {
                e.preventDefault();
                location.href = `slide-${current - 1}.html`;
            }
            break;
        case 'Home':
            e.preventDefault();
            location.href = 'slide-1.html';
            break;
        case 'End':
        case 'G':
            e.preventDefault();
            location.href = `slide-${total}.html`;
            break;
        case 'Escape':
            e.preventDefault();
            location.href = `../slide-${current}.html`;
            break;
        case 'o':
            e.preventDefault();
            location.href = '../overview.html';
            break;
    }
});
