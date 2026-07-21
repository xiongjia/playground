#!/usr/bin/env node
/**
 * gen-index.ts — Scan VIDEO_DL_ROOT and generate index.html + player.html
 *
 * Usage:
 *   npx tsx gen-index.ts
 *
 * Environment:
 *   VIDEO_DL_ROOT   — root directory to scan (required)
 *
 * Output:
 *   $VIDEO_DL_ROOT/index.html        — main page with nav tree + video grid
 *   $VIDEO_DL_ROOT/player.html       — native HTML5 video player page
 */

import * as fs from 'node:fs';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';

// ── helpers ──────────────────────────────────────────────────

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const MODULE_DIR = path.resolve(__dirname, '..');
const VIDEO_DL_ROOT = process.env.VIDEO_DL_ROOT || '';

function htmlEscape(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

interface VideoEntry {
  title: string;
  date: string;
  dir: string; // relative to VIDEO_DL_ROOT
  mp4: string; // relative path to video file
  thumb: string | null; // relative path to thumbnail
  vtts: { lang: string; label: string; file: string; }[];
  info: Record<string, unknown> | null;
}

interface CategoryGroup {
  name: string;
  type: 'channel' | 'playlist' | 'single';
  videos: VideoEntry[];
}

// ── ensure assets ────────────────────────────────────────────

// ── scan directory tree ──────────────────────────────────────

function parseLangFromFilename(filename: string): { lang: string; label: string; } | null {
  const match = filename.match(/\.([a-z]{2}(-[A-Z][a-z]+)?)\.vtt$/);
  if (!match) return null;

  const langMap: Record<string, string> = {
    en: 'English',
    'zh-Hans': '中文',
    'zh-Hant': '中文（繁體）',
    ja: '日本語',
    ko: '한국어',
    fr: 'Français',
    de: 'Deutsch',
    es: 'Español',
    ru: 'Русский',
    ar: 'العربية',
    pt: 'Português',
    it: 'Italiano',
    vi: 'Tiếng Việt',
    th: 'ไทย',
  };

  return { lang: match[1], label: langMap[match[1]] || match[1] };
}

function scanTree(root: string): CategoryGroup[] {
  const groups: CategoryGroup[] = [];
  const categories = ['channel', 'playlist', 'single'];

  for (const cat of categories) {
    const catDir = path.join(root, cat);
    if (!fs.existsSync(catDir)) continue;

    const entries = fs.readdirSync(catDir, { withFileTypes: true });
    for (const entry of entries) {
      if (!entry.isDirectory()) continue;

      const groupDir = path.join(catDir, entry.name);
      const videos = scanVideosInDir(groupDir, root);
      if (videos.length > 0) {
        groups.push({ name: entry.name, type: cat as 'channel' | 'playlist' | 'single', videos });
      }
    }
  }

  return groups.sort((a, b) => {
    const order = { channel: 0, playlist: 1, single: 2 };
    return order[a.type] - order[b.type] || a.name.localeCompare(b.name);
  });
}

function scanVideosInDir(dir: string, root: string): VideoEntry[] {
  const videos: VideoEntry[] = [];
  const entries = fs.readdirSync(dir, { withFileTypes: true });

  const videoDirs: string[] = [dir];
  for (const entry of entries) {
    if (entry.isDirectory() && !entry.name.startsWith('.')) {
      videoDirs.push(path.join(dir, entry.name));
    }
  }

  const seen = new Set<string>();
  for (const videoDir of videoDirs) {
    const files = fs.readdirSync(videoDir);

    const videoFile = files.find((f) =>
      f.endsWith('.mp4') || f.endsWith('.mkv') || f.endsWith('.webm')
    );
    if (!videoFile) continue;

    const mp4Rel = path.relative(root, path.join(videoDir, videoFile));
    if (seen.has(mp4Rel)) continue;
    seen.add(mp4Rel);

    const thumbFile = files.find((f) => f.endsWith('.thumbnail.jpg') || f.endsWith('.jpg'));
    const thumbRel = thumbFile ? path.relative(root, path.join(videoDir, thumbFile)) : null;
    const infoFile = files.find((f) => f.endsWith('.info.json'));
    let info: Record<string, unknown> | null = null;
    if (infoFile) {
      try {
        info = JSON.parse(fs.readFileSync(path.join(videoDir, infoFile), 'utf-8'));
      } catch { /* ignore */ }
    }

    const vtts = files
      .filter((f) => f.endsWith('.vtt'))
      .map((f) => {
        const parsed = parseLangFromFilename(f);
        return {
          lang: parsed?.lang || 'und',
          label: parsed?.label || 'Unknown',
          file: path.relative(root, path.join(videoDir, f)),
        };
      })
      .filter((v) => v.lang !== 'und');

    const baseName = path.basename(videoDir);
    const dateMatch = baseName.match(/^(\d{8})-(.*)/);
    const title = dateMatch ? dateMatch[2] : baseName.replace(/\.[^.]+$/, '');
    const date = dateMatch ? dateMatch[1] : '';

    videos.push({
      title,
      date,
      dir: path.relative(root, videoDir),
      mp4: mp4Rel,
      thumb: thumbRel,
      vtts,
      info,
    });
  }

  return videos.sort((a, b) => b.date.localeCompare(a.date));
}

// ── render templates ─────────────────────────────────────────

function renderIndex(groups: CategoryGroup[]): string {
  const navItems = groups.map((g) => {
    const icon = g.type === 'channel' ? '📺' : g.type === 'playlist' ? '📋' : '📹';
    return `<li class="nav-${g.type}"><a href="#" onclick="filterByGroup('${
      htmlEscape(g.name)
    }')">${icon} ${htmlEscape(g.name)}</a> <span class="count">${g.videos.length}</span></li>`;
  }).join('\n');

  const cards = groups.flatMap((g) =>
    g.videos.map((v) => {
      const thumbHtml = v.thumb
        ? `<img class="thumb" src="${
          htmlEscape(v.thumb)
        }" alt="" loading="lazy" onerror="this.style.display='none'">`
        : `<div class="thumb thumb-placeholder">🎬</div>`;
      const subCount = v.vtts.length > 0
        ? `<span class="sub-badge">${v.vtts.length} CC</span>`
        : '';
      const dateStr = v.date ? v.date.replace(/(\d{4})(\d{2})(\d{2})/, '$1-$2-$3') : '';
      return `<div class="card" data-group="${htmlEscape(g.name)}" data-title="${
        htmlEscape(v.title).toLowerCase()
      }">
        <a href="player.html?file=${encodeURIComponent(v.mp4)}${
        v.vtts.length > 0 ? '&langs=' + encodeURIComponent(v.vtts.map(x => x.lang).join(',')) : ''
      }">
          ${thumbHtml}
          <div class="card-body">
            <div class="card-title">${htmlEscape(v.title)}</div>
            <div class="card-meta">${htmlEscape(g.name)}${
        dateStr ? ` · ${dateStr}` : ''
      }${subCount}</div>
          </div>
        </a>
      </div>`;
    })
  ).join('\n');

  return `<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Video Library</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #0f0f0f; color: #eee; min-height: 100vh; }
.layout { display: flex; min-height: 100vh; }
.sidebar { width: 280px; background: #1a1a1a; padding: 20px; overflow-y: auto; flex-shrink: 0; border-right: 1px solid #2a2a2a; }
.sidebar h2 { font-size: 16px; margin-bottom: 16px; color: #aaa; text-transform: uppercase; letter-spacing: 1px; }
.sidebar ul { list-style: none; }
.sidebar li { padding: 8px 12px; border-radius: 6px; cursor: pointer; font-size: 14px; display: flex; justify-content: space-between; align-items: center; }
.sidebar li:hover { background: #2a2a2a; }
.sidebar li.active { background: #3a3a3a; color: #fff; }
.sidebar .count { font-size: 12px; color: #666; background: #2a2a2a; padding: 2px 8px; border-radius: 10px; }
.main { flex: 1; padding: 20px; }
.toolbar { margin-bottom: 20px; display: flex; gap: 12px; align-items: center; }
.toolbar input { flex: 1; max-width: 400px; padding: 10px 16px; border-radius: 8px; border: 1px solid #333; background: #1a1a1a; color: #eee; font-size: 14px; }
.toolbar input:focus { outline: none; border-color: #4a9eff; }
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 16px; }
.card { background: #1a1a1a; border-radius: 12px; overflow: hidden; transition: transform 0.15s; }
.card:hover { transform: scale(1.02); }
.card a { text-decoration: none; color: inherit; }
.thumb { width: 100%; aspect-ratio: 16/9; object-fit: cover; background: #2a2a2a; }
.thumb-placeholder { display: flex; align-items: center; justify-content: center; font-size: 40px; }
.card-body { padding: 12px; }
.card-title { font-size: 14px; font-weight: 500; line-height: 1.3; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
.card-meta { font-size: 12px; color: #888; margin-top: 6px; display: flex; gap: 8px; align-items: center; }
.sub-badge { background: #2a4a6a; color: #8ab4f8; padding: 1px 6px; border-radius: 4px; font-size: 11px; }
@media (max-width: 768px) { .sidebar { display: none; } .grid { grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); } }
</style>
</head>
<body>
<div class="layout">
<nav class="sidebar">
<h2>📂 Library</h2>
<ul>
<li class="active" onclick="filterByGroup('')"><a href="#">📌 All</a> <span class="count">${
    groups.reduce((s, g) => s + g.videos.length, 0)
  }</span></li>
${navItems}
</ul>
</nav>
<div class="main">
<div class="toolbar">
<input id="search" type="text" placeholder="Search videos..." oninput="filterCards()">
</div>
<div class="grid" id="grid">
${cards}
</div>
</div>
</div>
<script>
function filterByGroup(group) {
  document.querySelectorAll('.sidebar li').forEach(el => el.classList.remove('active'));
  if (group) event.currentTarget.closest('li').classList.add('active');
  else document.querySelector('.sidebar li:first-child').classList.add('active');
  window._filterGroup = group;
  filterCards();
}
function filterCards() {
  const q = document.getElementById('search').value.toLowerCase();
  const g = window._filterGroup || '';
  document.querySelectorAll('.card').forEach(c => {
    const matchGroup = !g || c.dataset.group === g;
    const matchTitle = !q || c.dataset.title.includes(q);
    c.style.display = matchGroup && matchTitle ? '' : 'none';
  });
}
</script>
</body>
</html>`;
}

function renderPlayer(): string {
  return `<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Player</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #000; color: #eee; height: 100vh; display: flex; flex-direction: column; overflow: hidden; }
.toolbar { padding: 12px 20px; background: #111; display: flex; align-items: center; gap: 16px; }
.toolbar a { color: #8ab4f8; text-decoration: none; font-size: 14px; }
.toolbar a:hover { text-decoration: underline; }
.toolbar .title { flex: 1; font-size: 14px; color: #aaa; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.player-container { flex: 1; display: flex; align-items: center; justify-content: center; background: #000; overflow: hidden; }
.player-container video { width: 100%; height: 100%; max-width: 100%; max-height: 100%; object-fit: contain; }
.player-container video::cue { font-size: 0.7vw; background: rgba(0,0,0,0.5); }
</style>
</head>
<body>
<div class="toolbar">
<a href="index.html">← Back</a>
<span class="title" id="videoTitle">Loading...</span>
</div>
<div class="player-container">
<video id="player" controls preload="auto"></video>
</div>
<script>
const params = new URLSearchParams(window.location.search);
const file = params.get('file');
if (!file) {
  document.getElementById('videoTitle').textContent = 'No file specified';
  document.getElementById('player').style.display = 'none';
} else {
  var src = window.location.origin + '/' + file.split('/').map(function(s) { return encodeURIComponent(s); }).join('/');
  var ext = file.split('.').pop().toLowerCase();
  var typeMap = { mp4: 'video/mp4', mkv: 'video/x-matroska', webm: 'video/webm' };

  var base = file.replace(/\\.[^.]+$/, '');
  var langs = ['en', 'zh-Hans', 'ja', 'ko'];
  var labels = { en: 'English', 'zh-Hans': '中文', ja: '日本語', ko: '한국어' };
  langs.forEach(function(lang) {
    var vttUrl = window.location.origin + '/' + (base + '.' + lang + '.vtt').split('/').map(function(s) { return encodeURIComponent(s); }).join('/');
    var track = document.createElement('track');
    track.kind = 'subtitles';
    track.src = vttUrl;
    track.srclang = lang;
    track.label = labels[lang] || lang;
    if (lang === 'en') track.default = true;
    document.getElementById('player').appendChild(track);
  });

  var source = document.createElement('source');
  source.src = src;
  source.type = typeMap[ext] || 'video/mp4';
  document.getElementById('player').appendChild(source);
  var langs = ['en', 'zh-Hans', 'ja', 'ko'];
  var labels = { en: 'English', 'zh-Hans': '中文', ja: '日本語', ko: '한국어' };
  langs.forEach(function(lang) {
    var vttUrl = window.location.origin + '/' + (base + '.' + lang + '.vtt').split('/').map(function(s) { return encodeURIComponent(s); }).join('/');
    var track = document.createElement('track');
    track.kind = 'subtitles';
    track.src = vttUrl;
    track.srclang = lang;
    track.label = labels[lang] || lang;
    if (lang === 'en') track.default = true;
    document.getElementById('player').appendChild(track);
  });

  document.getElementById('videoTitle').textContent = file.split('/').pop().replace(/\\.[^.]+$/, '');
  // Center subtitles by resetting VTT cue align after track loads
  var player = document.getElementById('player');
  player.addEventListener('loadstart', function() {
    function centerCues(track) {
      for (var k = 0; k < track.cues.length; k++) {
        var c = track.cues[k];
        c.align = 'center';
        c.size = 80;
        c.position = 50;
      }
    }
    for (var i = 0; i < player.textTracks.length; i++) {
      var t = player.textTracks[i];
      if (t.kind === 'subtitles') {
        if (t.cues && t.cues.length > 0) {
          centerCues(t);
        } else {
          t.addEventListener('cuechange', function() { centerCues(this); });
        }
      }
    }
  });
  document.getElementById('player').play();
}
</script>
</body>
</html>`;
}

// ── main ──────────────────────────────────────────────────────

function main(): void {
  if (!VIDEO_DL_ROOT || !fs.existsSync(VIDEO_DL_ROOT)) {
    console.error('✗ VIDEO_DL_ROOT is not set or does not exist');
    console.error('  Usage: VIDEO_DL_ROOT=/path/to/media npx tsx gen-index.ts');
    process.exit(1);
  }

  console.log('📂 Scanning:', VIDEO_DL_ROOT);

  console.log('  Scanning directory tree ...');
  const groups = scanTree(VIDEO_DL_ROOT);
  const totalVideos = groups.reduce((s, g) => s + g.videos.length, 0);
  console.log('  Found', totalVideos, 'videos in', groups.length, 'groups');

  console.log('  Generating index.html ...');
  const indexHtml = renderIndex(groups);
  fs.writeFileSync(path.join(VIDEO_DL_ROOT, 'index.html'), indexHtml, 'utf-8');

  console.log('  Generating player.html ...');
  const playerHtml = renderPlayer();
  fs.writeFileSync(path.join(VIDEO_DL_ROOT, 'player.html'), playerHtml, 'utf-8');

  console.log('✅ Done! Open:', path.join(VIDEO_DL_ROOT, 'index.html'));
  console.log('   Or serve with: cargo run --bin static-server --', VIDEO_DL_ROOT);
}

main();
