<template>
  <canvas ref="canvasEl" class="ember-canvas" aria-hidden="true" />
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";

const canvasEl = ref<HTMLCanvasElement | null>(null);
let rafId = 0;
let resizeObserver: ResizeObserver | null = null;
let themeObserver: MutationObserver | null = null;
let themeColors: string[] = ["#d4520a", "#c9972a", "#ff7b2e"];

interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  size: number;
  opacity: number;
  life: number;
  maxLife: number;
  colorIndex: number;
}

const PARTICLE_COUNT = 120;
const particles: Particle[] = [];

function readThemeColors() {
  const s = getComputedStyle(document.documentElement);
  const p = s.getPropertyValue("--accent-primary").trim();
  const sec = s.getPropertyValue("--accent-secondary").trim();
  const rune = s.getPropertyValue("--rune-glow").trim();
  themeColors = [p || "#d4520a", sec || "#c9972a", rune || "#ff7b2e"];
}

function randomParticle(w: number, h: number, partial = false): Particle {
  return {
    x: Math.random() * w,
    y: partial ? Math.random() * h : h + Math.random() * 20,
    vx: (Math.random() - 0.5) * 0.35,
    vy: -(0.25 + Math.random() * 0.65),
    size: 0.8 + Math.random() * 2.2,
    opacity: 0,
    life: 0,
    maxLife: 200 + Math.random() * 260,
    colorIndex: Math.floor(Math.random() * 3),
  };
}

function initParticles(w: number, h: number) {
  particles.length = 0;
  for (let i = 0; i < PARTICLE_COUNT; i++) {
    const p = randomParticle(w, h, true);
    p.life = Math.random() * p.maxLife;
    particles.push(p);
  }
}

function tick() {
  const canvas = canvasEl.value;
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  const w = canvas.width;
  const h = canvas.height;

  ctx.clearRect(0, 0, w, h);

  for (const p of particles) {
    p.life++;
    p.x += p.vx + Math.sin(p.life * 0.018 + p.y * 0.003) * 0.18;
    p.y += p.vy;

    const progress = p.life / p.maxLife;
    if (progress < 0.12) {
      p.opacity = progress / 0.12;
    } else if (progress > 0.78) {
      p.opacity = 1 - (progress - 0.78) / 0.22;
    } else {
      p.opacity = 1;
    }

    if (p.life >= p.maxLife || p.y < -12) {
      Object.assign(p, randomParticle(w, h));
    }

    ctx.save();
    ctx.globalAlpha = p.opacity * 0.6;
    ctx.fillStyle = themeColors[p.colorIndex] ?? themeColors[0];
    ctx.beginPath();
    ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
    ctx.fill();
    ctx.restore();
  }

  rafId = requestAnimationFrame(tick);
}

onMounted(() => {
  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

  const canvas = canvasEl.value!;

  readThemeColors();

  function resize() {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    initParticles(canvas.width, canvas.height);
  }

  resize();
  tick();

  resizeObserver = new ResizeObserver(resize);
  resizeObserver.observe(document.body);

  // Re-read colors when theme attribute changes
  themeObserver = new MutationObserver(readThemeColors);
  const root = document.getElementById("umbra-root") ?? document.documentElement;
  themeObserver.observe(root, { attributes: true, attributeFilter: ["data-theme"] });
});

onUnmounted(() => {
  cancelAnimationFrame(rafId);
  resizeObserver?.disconnect();
  themeObserver?.disconnect();
});
</script>

<style scoped>
.ember-canvas {
  position: fixed;
  inset: 0;
  z-index: 0;
  pointer-events: none;
}
</style>
