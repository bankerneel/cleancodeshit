(function () {
  'use strict';
  var reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  /* ---------- loader ---------- */
  var loader = document.getElementById('loader');
  function endLoader() {
    if (!loader) return;
    var l = loader; loader = null;
    l.classList.add('done');
    setTimeout(function () { l.remove(); }, 450);
  }
  if (reduce) endLoader();
  else {
    setTimeout(endLoader, 1350);
    if (loader) loader.addEventListener('click', endLoader);
    document.addEventListener('keydown', endLoader, { once: true });
  }

  /* ---------- items registered by the CLI ---------- */
  var CLI_ITEMS = [
    ["npm", "Package managers", "safe", "npm cache"],
    ["npx", "Package managers", "safe", "npx downloaded tools"],
    ["pnpm", "Package managers", "safe", "pnpm store (packages no project uses)"],
    ["yarn", "Package managers", "safe", "Yarn cache"],
    ["bun", "Package managers", "safe", "bun install cache"],
    ["uv", "Package managers", "safe", "uv cache (unused entries)"],
    ["pip", "Package managers", "safe", "pip download cache"],
    ["brew", "Package managers", "caution", "Homebrew downloads and old versions"],
    ["xcode", "Build and tool caches", "safe", "Xcode DerivedData"],
    ["playwright", "Build and tool caches", "caution", "Playwright browsers"],
    ["electron", "Build and tool caches", "safe", "Electron and electron-builder downloads"],
    ["nodegyp", "Build and tool caches", "safe", "node-gyp headers"],
    ["typescript", "Build and tool caches", "safe", "TypeScript type cache"],
    ["go", "Build and tool caches", "safe", "Go build cache"],
    ["cargo", "Build and tool caches", "safe", "Cargo crate downloads"],
    ["cocoapods", "Build and tool caches", "safe", "CocoaPods cache"],
    ["gradle", "Build and tool caches", "caution", "Gradle caches"],
    ["precommit", "Build and tool caches", "safe", "pre-commit environments"],
    ["dk_all", "Docker", "danger", "ALL containers, running ones included"],
    ["dk_stopped", "Docker", "danger", "Stopped containers"],
    ["dk_custom", "Docker", "danger", "Images matching your configuration"],
    ["dk_images", "Docker", "danger", "Unused images"],
    ["dk_build", "Docker", "safe", "Build cache"],
    ["dk_net", "Docker", "safe", "Unused networks"],
    ["dk_volumes", "Docker", "danger", "Unused volumes (may hold databases)"],
    ["claude_debug", "AI coding agents", "safe", "Claude Code debug logs"],
    ["claude_history", "AI coding agents", "danger", "Claude Code session transcripts"],
    ["codex_logs", "AI coding agents", "safe", "Codex CLI logs"],
    ["codex_history", "AI coding agents", "danger", "Codex CLI session transcripts"],
    ["gemini_history", "AI coding agents", "danger", "Gemini CLI session transcripts"],
    ["copilot_logs", "AI coding agents", "safe", "GitHub Copilot CLI logs"],
    ["opencode_logs", "AI coding agents", "safe", "OpenCode logs"],
    ["vscode", "AI coding agents", "safe", "VS Code logs and caches"],
    ["cursor", "AI coding agents", "safe", "Cursor logs and caches (macOS)"],
    ["windsurf", "AI coding agents", "safe", "Windsurf and Devin Desktop logs"],
    ["puppeteer", "AI coding agents", "caution", "Puppeteer browser downloads"],
    ["hf_hub", "Local models", "caution", "Hugging Face Hub models"],
    ["ollama", "Local models", "caution", "Ollama models"],
    ["torch_hub", "Local models", "caution", "PyTorch Hub cache"],
    ["lm_studio", "Local models", "caution", "LM Studio GGUF models"],
    ["stale", "Projects", "caution", "Stale project folders"]
  ];
  var CLI_LABELS = Object.fromEntries(CLI_ITEMS.map(function (item) { return [item[0], item[3]]; }));
  var binNames = ['Package managers', 'Build and tool caches', 'AI coding agents', 'Docker', 'Local models', 'Projects'];
  var binsEl = document.getElementById('bins');
  binNames.forEach(function (group) {
    var items = CLI_ITEMS.filter(function (item) { return item[1] === group; });
    var stale = group === 'Projects';
    var bin = document.createElement('div');
    bin.className = 'bin' + (group === 'AI coding agents' || group === 'Local models' ? ' hot' : '');
    var heading = document.createElement('h3');
    heading.textContent = stale ? 'Stale project folders' : group;
    var count = document.createElement('small');
    count.textContent = stale ? 'opt-in' : items.length;
    heading.appendChild(count);
    var list = document.createElement('ul');
    items.forEach(function (item) {
      var row = document.createElement('li');
      row.dataset.itemId = item[0];
      var label = document.createElement('span');
      label.textContent = item[3];
      var badge = document.createElement('span');
      badge.className = 'r ' + item[2];
      badge.textContent = item[2];
      row.append(label, badge);
      list.appendChild(row);
    });
    bin.append(heading, list);
    if (stale) {
      var note = document.createElement('p');
      note.className = 'note';
      note.textContent = 'Opt-in only, with --in DIR. Never in a preset.';
      bin.appendChild(note);
    }
    binsEl.appendChild(bin);
  });

  /* ---------- disk meter ---------- */
  var usedEl = document.getElementById('usedGB'), junkSeg = document.getElementById('junkSeg'), disk = document.getElementById('disk');
  var SAMPLE_BASE = 1200, SAMPLE_CAPACITY = 2560;
  var HERO_CHIP_BUDGET = 200, FULL_CHIP_BUDGET = 730;
  var shown = (SAMPLE_BASE + HERO_CHIP_BUDGET) / 10, tweenId = 0;

  function sumTenths(items) {
    return items.reduce(function (sum, item) { return sum + item.tenths; }, 0);
  }

  function setUsed(tenths) {
    var v = tenths / 10;
    junkSeg.style.width = ((tenths - SAMPLE_BASE) / SAMPLE_CAPACITY * 100) + '%';
    disk.setAttribute('aria-label', 'Sample disk: ' + v.toFixed(1) + ' GB used of 256 GB');
    disk.classList.toggle('full', tenths / SAMPLE_CAPACITY > .95);
    var from = shown, start = performance.now(), id = ++tweenId;
    if (reduce) { shown = v; usedEl.textContent = v.toFixed(1); return; }
    (function step(now) {
      if (id !== tweenId) return;
      var k = Math.min(1, (now - start) / 700), e = 1 - Math.pow(1 - k, 3);
      shown = from + (v - from) * e;
      usedEl.textContent = shown.toFixed(1);
      if (k < 1) requestAnimationFrame(step);
    })(start);
  }

  /* ---------- toast ---------- */
  var toast = document.getElementById('toast');
  function showToast(on) { toast.classList.toggle('show', !!on); }
  document.getElementById('toastX').addEventListener('click', function () { showToast(false); });


  function createPile(opts) {
    'use strict';
    var canvas = opts.canvas;
    var reduce = !!opts.reduce;
    var catalog = opts.catalog || [];
    var slabDef = opts.slab || null;
    var onChange = typeof opts.onChange === 'function' ? opts.onChange : function () {};
    var onDanger = typeof opts.onDanger === 'function' ? opts.onDanger : function () {};

    var INK = '#111216', PORCELAIN = '#F3F5F8', YELLOW = '#FFC83D', DIM = '#8E919B', RED = '#E5383B';
    var FONT = '700 12px "Plex Mono", ui-monospace, monospace';
    var MICRO = '700 8px "Plex Mono", ui-monospace, monospace';
    var COL = 8, GLOW_MS = 200;
    var ctx = canvas && canvas.getContext ? canvas.getContext('2d') : null;

    var noop = {
      setHighlights: function () {}, setPreview: function () {}, flush: function () { return []; },
      reset: function () {}, getItems: function () { return []; }
    };
    if (!ctx) return noop;

    var W = 0, H = 0, DPR = 1, capH = 0, bandTop = 0, cols = 1, heights = new Float32Array(1);
    var items = [], falling = [], dust = [], vortex = null, pendingDone = null;
    var seq = 0, pickSeq = 0, bag = [], pools = { safe: [], caution: [], danger: [], stale: [] };
    var highlighted = Object.create(null), preview = null;
    var targetFill = .15, spawnBudget = 0, slabPlaced = false;
    var running = false, raf = 0, last = 0, covered = false, resizeT = 0, lastScrollY = 0;
    var stages = [], stageIndex = -1, flushTop = Infinity;
    var opaqueStart = document.querySelector('.opaque');
    var sprites = new Map(), glowUntil = 0;
    var onSlabLand = opts.onSlabLand;
    var measureCtx = document.createElement('canvas').getContext('2d');

    function rnd(a, b) { return a + Math.random() * (b - a); }
    function clamp(v, a, b) { return v < a ? a : v > b ? b : v; }
    function rr(c, x, y, w, h, r) {
      c.beginPath();
      if (c.roundRect) c.roundRect(x, y, w, h, r); else c.rect(x, y, w, h);
    }
    function textW(s) {
      measureCtx.font = FONT;
      return measureCtx.measureText(s).width;
    }

    catalog.forEach(function (d) {
      if (d && d.label && (d.risk === 'safe' || d.risk === 'caution' || d.risk === 'danger' || d.risk === 'stale')) {
        pools[d.risk].push(d);
      }
    });

    /* ---------- pre-rendered sprites (one per chip, no text in the frame loop) ---------- */
    function spriteFor(def, isSlab) {
      var label = def.label || '', risk = def.risk || 'safe', gb = +def.gb || 0, w, h;
      if (isSlab) {
        w = Math.round(W < 640 ? clamp(W * .55, 200, 360) : clamp(W * .68, 230, 360));
        h = Math.round(clamp(H * (W < 640 ? .073 : .104), 58, 92));
      } else {
        h = Math.round(clamp(22 + Math.log2(1 + gb) * 1.6, 22, 34)) - (W < 640 ? 5 : 0);
        var extra = risk === 'danger' ? (W < 640 ? 59 : 96) : risk === 'stale' ? (W < 640 ? 38 : 46) : 0;
        w = Math.round(clamp(textW(label) + 20 + extra, W < 640 ? 60 : 66, Math.max(80, W - 24)));
      }
      var key = label + '/' + w + '/' + h + '/' + DPR;
      if (sprites.has(key)) return sprites.get(key);
      var c = document.createElement('canvas');
      c.width = Math.ceil(w * DPR);
      c.height = Math.ceil(h * DPR);
      var g = c.getContext('2d');
      g.setTransform(DPR, 0, 0, DPR, 0, 0);
      g.textBaseline = 'middle';

      if (isSlab) {
        g.fillStyle = YELLOW; rr(g, 0, 0, w, h, 8); g.fill();
        g.save(); rr(g, 0, 0, w, h, 8); g.clip();
        g.beginPath(); g.rect(0, 0, 44, h); g.clip();
        g.fillStyle = INK;
        for (var i = -h; i < 52; i += 14) {
          g.beginPath(); g.moveTo(i, h); g.lineTo(i + 7, h); g.lineTo(i + 7 + h, 0); g.lineTo(i + h, 0); g.fill();
        }
        g.restore();
        g.fillStyle = INK; g.font = FONT; g.fillText(label, 56, h * 0.3);
        g.font = '700 ' + Math.round(h * 0.43) + 'px "Archivo Black", Impact, sans-serif';
        g.fillText(gb + ' GB', 56, h * 0.7);
        g.fillStyle = INK; rr(g, w - 64, 8, 56, 18, 3); g.fill();
        g.fillStyle = PORCELAIN; g.font = MICRO; g.fillText('sample', w - 58, 17);
        var slabSprite = { node: c, w: w, h: h };
        sprites.set(key, slabSprite);
        return slabSprite;
      }

      if (risk === 'safe') {
        g.fillStyle = PORCELAIN; rr(g, 0, 0, w, h, 4); g.fill();
        g.strokeStyle = 'rgba(17,18,22,.35)'; g.lineWidth = 1; rr(g, .5, .5, w - 1, h - 1, 4); g.stroke();
        g.fillStyle = INK; g.font = FONT; g.fillText(label, 8, h / 2 + .5);
      } else if (risk === 'caution') {
        g.fillStyle = YELLOW; rr(g, 0, 0, w, h, 4); g.fill();
        g.fillStyle = INK; g.font = FONT; g.fillText(label, 8, h / 2 + .5);
      } else if (risk === 'danger') {
        g.fillStyle = YELLOW; rr(g, 0, 0, w, h, 4); g.fill();
        g.save(); rr(g, 0, 0, w, h, 4); g.clip();
        g.fillStyle = INK;
        for (var j = -h; j < w; j += 16) {
          g.beginPath(); g.moveTo(j, h); g.lineTo(j + 8, h); g.lineTo(j + 8 + h, 0); g.lineTo(j + h, 0); g.fill();
        }
        g.restore();
        g.font = FONT;
        g.fillStyle = INK; rr(g, 4, h / 2 - 9, textW(label) + 8, 18, 2); g.fill();
        g.fillStyle = PORCELAIN; g.fillText(label, 8, h / 2 + 1);
        var tab = W < 640 ? 18 : 28, tagX = w - tab - 42;
        g.fillStyle = INK; rr(g, tagX, h / 2 - 8, 42, 16, 2); g.fill();
        g.fillStyle = RED; g.font = MICRO; g.fillText('needs yes', tagX + 2, h / 2 + .5);
        g.fillStyle = RED; rr(g, w - tab, 0, tab, h, 4); g.fill();
        g.fillStyle = INK; g.font = '700 15px "Plex Mono", monospace'; g.textAlign = 'center';
        g.fillText('!', w - tab / 2, h / 2 + .5); g.textAlign = 'left';
        g.strokeStyle = RED; g.lineWidth = 2; rr(g, 1, 1, w - 2, h - 2, 4); g.stroke();
      } else {
        g.fillStyle = INK; rr(g, 0, 0, w, h, 4); g.fill();
        g.strokeStyle = DIM; g.lineWidth = 1; g.setLineDash([3, 2]); rr(g, .5, .5, w - 1, h - 1, 4); g.stroke();
        g.setLineDash([]);
        g.fillStyle = PORCELAIN; g.font = FONT; g.fillText(label, 8, h / 2 + .5);
        g.fillStyle = DIM; g.font = MICRO; g.fillText('opt-in', w - 44, h / 2 + .5);
      }
      var chipSprite = { node: c, w: w, h: h };
      sprites.set(key, chipSprite);
      return chipSprite;
    }

    function regularCapacity() { return W < 640 ? 18 : 30; }

    function maxItems() {
      return Math.round(regularCapacity() * targetFill);
    }

    function newItem(def, isSlab, copies) {
      var sprite = spriteFor(def, isSlab);
      var widthScale = isSlab ? 1 : [1, .9, 1.1][copies];
      return {
        id: 'p' + (++seq), label: def.label, risk: def.risk, tenths: isSlab ? def.gb * 10 : 0, slab: isSlab,
        def: def, w: sprite.w * widthScale, h: sprite.h, sprite: sprite.node, widthScale: widthScale,
        x: 0, y: 0, a: 0, vy: 0, va: 0, targetY: 0, ta: 0, fx: null,
        glowFrom: 0, glowTo: 0, glowStart: 0
      };
    }

    /* Reserve each landing position as soon as its piece begins to fall. */
    function spanMax(x, w) {
      var left = Math.max(0, Math.floor(x / COL));
      var right = Math.min(cols - 1, Math.ceil((x + w) / COL) - 1);
      var max = 0;
      for (var c = left; c <= right; c++) max = Math.max(max, heights[c]);
      return max;
    }

    function lowestSpan(w) {
      var best = Infinity, bx = 0;
      for (var x = 0; x < W - w + COL; x += COL) {
        var candidate = Math.min(x, W - w);
        var h = spanMax(candidate, w);
        if (h < best) { best = h; bx = candidate; }
      }
      return { x: bx, height: best };
    }

    function plot(it, x, base) {
      it.x = x;
      it.targetY = H - base - it.h;
      it.ta = reduce || !base ? 0 : rnd(-.055, .055);
      var first = Math.floor(x / COL), lastCol = Math.min(cols - 1, Math.ceil((x + it.w) / COL) - 1);
      for (var c = first; c <= lastCol; c++) heights[c] = base + it.h + (W < 640 ? 1 : 2);
    }

    function moundLimit() { return Math.floor(capH * .90); }

    function spawnX(it) {
      var center = Math.random() < .64 ? .29 : .71;
      var jitter = rnd(-.055, .055) * W;
      return (W * center) - it.w / 2 + jitter;
    }

    function placeChip(it, keepRoom) {
      var preferred = clamp(it.x, 0, W - it.w);
      var ceiling = keepRoom || slabPlaced ? moundLimit() : capH;
      var w = it.w, h = it.h;
      var initial = { x: preferred, height: spanMax(preferred, w) };

      // A chip can slip once if the neighbour is much lower than its landing.
      var left = Math.max(0, preferred - w * .6);
      var right = Math.min(W - w, preferred + w * .6);
      var leftHeight = spanMax(left, w), rightHeight = spanMax(right, w);
      var downhill = leftHeight <= rightHeight ? { x: left, height: leftHeight } : { x: right, height: rightHeight };
      if (initial.height - downhill.height > h * 1.5) initial = downhill;

      function tryLanding(site) {
        if (site.height + h > ceiling) return false;
        if (!keepRoom) {
          plot(it, site.x, site.height);
          return true;
        }
        var start = Math.floor(site.x / COL);
        var end = Math.min(cols, Math.ceil((site.x + w) / COL));
        var previous = heights.slice(start, end);
        plot(it, site.x, site.height);
        if (lowestSpan(keepRoom.w).height + keepRoom.h <= ceiling) return true;
        heights.set(previous, start);
        return false;
      }

      var phone = W < 640;
      if ((!phone || initial.height <= lowestSpan(w).height + h * .7) && tryLanding(initial)) return true;
      // Pick a new landing before spawning if the planned columns are full.
      var sites = [];
      for (var x = 0; x < W - w + COL; x += COL) {
        var candidate = Math.min(x, W - w);
        var base = spanMax(candidate, w);
        if (base + h <= ceiling) sites.push({ x: candidate, height: base });
      }
      sites.sort(function (a, b) {
        var distanceWeight = phone ? .13 : 1;
        return (Math.abs(a.x - preferred) - Math.abs(b.x - preferred)) * distanceWeight +
               (a.height - b.height) * 1.6;
      });
      return sites.some(tryLanding);
    }

    function slabSize() {
      return slabDef && !slabPlaced ? spriteFor(slabDef, true) : null;
    }

    function snap(it) {
      it.y = it.targetY;
      it.a = it.ta;
      it.vy = 0;
      it.va = 0;
      it.fx = null;
    }

    function repack(list, animate) {
      heights = new Float32Array(cols);
      list.forEach(function (it) {
        var x = it.x, y = it.y;
        if (it.slab) {
          var span = lowestSpan(it.w);
          plot(it, span.x, Math.min(span.height, capH - it.h));
        } else if (!placeChip(it, null)) {
          var span = lowestSpan(it.w);
          plot(it, span.x, capH - it.h);
        }
        if (!animate) { snap(it); return; }
        it.fx = clamp(x, 0, W - it.w);
        it.y = Math.min(y, it.targetY - rnd(12, 24));
        it.vy = 75;
        it.va = 0;
      });
    }

    /* ---------- staged spawning ---------- */
    function getItems() { return items.concat(falling); }

    function pickChip(used, reserved) {
      if (!bag.length) bag = ['safe', 'caution', 'danger', 'stale', 'safe', 'danger', 'caution'];
      var preferred = bag.shift();
      var offset = pickSeq++;
      var risks = [preferred].concat(['safe', 'caution', 'stale', 'danger'].filter(function (risk) {
        return risk !== preferred;
      }));
      for (var r = 0; r < risks.length; r++) {
        var pool = pools[risks[r]], start = offset % pool.length;
        var order = pool.slice(start).concat(pool.slice(0, start));
        order.sort(function (a, b) {
          var aa = (used.get(a.label) || 0) * 2 - (highlighted[a.label] ? 3 : 0);
          var bb = (used.get(b.label) || 0) * 2 - (highlighted[b.label] ? 3 : 0);
          return aa - bb;
        });
        for (var i = 0; i < order.length; i++) {
          var def = order[i];
          var copies = used.get(def.label) || 0;
          if (copies >= 3) continue;
          var it = newItem(def, false, copies);
          it.x = spawnX(it);
          if (placeChip(it, reserved)) return it;
        }
      }
      bag.unshift(preferred);
      return null;
    }

    function addChip(it, instant) {
      aimGlow(it, performance.now());
      if (instant) {
        snap(it);
        items.push(it);
        return;
      }
      it.y = -it.h - rnd(0, 80);
      it.a = rnd(-.24, .24);
      it.vy = rnd(90, 170);
      it.va = rnd(-.8, .8);
      falling.push(it);
    }

    function grow(limit, instant, hero) {
      var live = getItems();
      var regular = live.filter(function (it) { return !it.slab; });
      var used = new Map();
      regular.forEach(function (it) { used.set(it.label, (used.get(it.label) || 0) + 1); });
      var reserved = slabSize();
      var needed = Math.min(limit, maxItems() - regular.length);
      var count = 0;
      var budget = (hero ? HERO_CHIP_BUDGET : FULL_CHIP_BUDGET) - sumTenths(regular);
      var slots = (hero ? maxItems() : regularCapacity()) - regular.length;
      while (count < needed && budget > 0) {
        var it = pickChip(used, reserved);
        if (!it) break;
        // Adjust each chip around the remaining share using its catalogue size.
        var average = budget / slots;
        var weight = .76 + Math.log2(1 + it.def.gb) * .105;
        it.tenths = slots === 1 ? budget : clamp(Math.round(average * weight), 1, Math.max(1, budget - slots + 1));
        budget -= it.tenths;
        slots--;
        used.set(it.label, (used.get(it.label) || 0) + 1);
        addChip(it, instant);
        count++;
      }
      return count;
    }

    function dropSlab() {
      if (!slabDef || slabPlaced) return false;
      var it = newItem(slabDef, true, 0);
      var span = lowestSpan(it.w);
      if (span.height + it.h > moundLimit()) return false;
      plot(it, span.x, span.height);
      slabPlaced = true;
      addChip(it, reduce);
      if (reduce && onSlabLand) onSlabLand();
      return true;
    }

    /* ---------- drawing ---------- */
    function glowAt(it, now) {
      if (it.glowFrom === it.glowTo) return it.glowTo;
      var progress = clamp((now - it.glowStart) / GLOW_MS, 0, 1);
      return it.glowFrom + (it.glowTo - it.glowFrom) * progress;
    }

    function aimGlow(it, now) {
      var target = !preview && highlighted[it.label] ? 1 : 0;
      if (it.glowTo === target) return;
      it.glowFrom = reduce ? target : glowAt(it, now);
      it.glowTo = target;
      it.glowStart = now;
      if (!reduce) glowUntil = now + GLOW_MS;
    }

    function applyGlow() {
      var now = performance.now();
      getItems().forEach(function (it) { aimGlow(it, now); });
      draw(now);
      kick();
    }

    function drawItem(it, x, y, angle, scale, alpha, glow) {
      ctx.save();
      ctx.globalAlpha = alpha;
      ctx.translate(x + it.w / 2, y + it.h / 2);
      if (angle) ctx.rotate(angle);
      if (scale !== 1) ctx.scale(scale, scale);
      ctx.drawImage(it.sprite, -it.w / 2, -it.h / 2, it.w, it.h);
      if (glow > 0) {
        rr(ctx, -it.w / 2 - 1, -it.h / 2 - 1, it.w + 2, it.h + 2, 4);
        ctx.strokeStyle = YELLOW;
        ctx.globalAlpha = .20 * glow;
        ctx.lineWidth = 7;
        ctx.stroke();
        ctx.globalAlpha = glow;
        ctx.lineWidth = 2;
        ctx.stroke();
      }
      ctx.restore();
    }

    function drawChip(it, now) {
      var bright = preview && it.risk !== 'danger' && preview(it);
      var alpha = preview ? (bright ? 1 : .18)
        : it.risk === 'danger' || it.slab ? .9 : it.risk === 'caution' ? .72 : .6;
      drawItem(it, it.fx == null ? it.x : it.fx, it.y, it.a, 1, alpha, glowAt(it, now));
    }

    function drawSpiral(v, fade) {
      var reach = Math.min(W * .5, H * .42);
      for (var arm = 0; arm < 3; arm++) {
        ctx.beginPath();
        for (var i = 0; i <= 28; i++) {
          var t = i / 28, radial = 7 + t * t * reach;
          var angle = v.t * 6 + arm * (2 * Math.PI / 3) - t * 3.3;
          var x = v.cx + Math.cos(angle) * radial, y = v.cy + Math.sin(angle) * radial * .62;
          if (i) ctx.lineTo(x, y); else ctx.moveTo(x, y);
        }
        ctx.globalAlpha = fade * (arm === 0 ? .62 : .30);
        ctx.strokeStyle = arm === 0 ? YELLOW : PORCELAIN;
        ctx.lineWidth = 1.5;
        ctx.stroke();
      }
      ctx.globalAlpha = 1;
    }

    function burst(it) {
      if (reduce) return;
      var count = W < 640 ? 20 : 24;
      for (var i = 0; i < count; i++) {
        dust.push({ x: it.x + rnd(8, it.w - 8), y: it.y + it.h * .9, vx: rnd(-90, 90), vy: rnd(-110, -30),
          s: rnd(1.5, 3.5), life: 1, c: i % 3 ? PORCELAIN : YELLOW });
      }
    }

    function draw(now) {
      if (!W || !H) return;
      ctx.clearRect(0, 0, W, H);
      for (var i = 0; i < items.length; i++) drawChip(items[i], now);
      for (var i = 0; i < falling.length; i++) drawChip(falling[i], now);
      for (var i = 0; i < dust.length; i++) {
        var p = dust[i];
        ctx.globalAlpha = Math.max(0, p.life);
        ctx.fillStyle = p.c;
        ctx.fillRect(p.x, p.y, p.s, p.s);
      }
      ctx.globalAlpha = 1;
      if (!vortex) return;
      var v = vortex;
      drawSpiral(v, Math.min(1, v.t / .18) * clamp((v.end - v.t) / .25, 0, 1));
      for (var i = 0; i < v.parts.length; i++) {
        var part = v.parts[i], it = part.it;
        if (part.k < 1) drawItem(it, part.x, part.y, part.angle, part.scale, part.alpha, 0);
      }
    }

    /* ---------- animation ---------- */
    function active(now) {
      return falling.length || dust.length || vortex || now < glowUntil;
    }

    function settleSurvivors() {
      var survivors = getItems();
      repack(survivors, true);
      items = [];
      falling = survivors;
    }

    function tick(now) {
      var dt = Math.min(.05, (now - last) / 1000 || .016);
      last = now;
      for (var i = falling.length - 1; i >= 0; i--) {
        var it = falling[i];
        it.vy += 1650 * dt;
        it.y += it.vy * dt;
        it.a += (it.ta - it.a) * Math.min(1, dt * 3) + it.va * dt;
        if (it.fx != null) it.fx += (it.x - it.fx) * Math.min(1, dt * 13);
        if (it.y >= it.targetY) {
          snap(it);
          falling.splice(i, 1);
          items.push(it);
          if (it.slab && !it.landed) {
            it.landed = true;
            burst(it);
            if (onSlabLand && lastScrollY < flushTop) onSlabLand();
          }
        }
      }
      for (var i = dust.length - 1; i >= 0; i--) {
        var p = dust[i];
        p.vy += 520 * dt; p.x += p.vx * dt; p.y += p.vy * dt; p.life -= dt * 1.3;
        if (p.life <= 0) dust.splice(i, 1);
      }
      if (vortex) {
        var v = vortex;
        v.t += dt;
        v.parts.forEach(function (part) {
          var k = (v.t - part.delay) / v.travel;
          part.k = k;
          if (k <= 0 || k >= 1) return;
          var distance = part.distance * (1 - k * k);
          part.theta += dt * (1.65 + 405 / (distance + 56));
          part.x = v.cx + Math.cos(part.theta) * distance - part.it.w / 2;
          part.y = v.cy + Math.sin(part.theta) * distance * .62 - part.it.h / 2;
          part.angle = part.it.a + k * part.theta * .7;
          part.scale = 1 - .90 * k * k;
          part.alpha = k > .82 ? (1 - k) / .18 : 1;
        });
        if (v.t >= v.end) {
          vortex = null;
          settleSurvivors();
        }
      }
      draw(now);
      if (active(now)) {
        raf = requestAnimationFrame(tick);
      } else {
        running = false;
        raf = 0;
        if (pendingDone) {
          var finish = pendingDone;
          pendingDone = null;
          finish();
        }
      }
    }

    function kick() {
      var now = performance.now();
      if (reduce || covered || document.hidden || running || !active(now)) return;
      running = true;
      last = now;
      raf = requestAnimationFrame(tick);
    }

    function stop() {
      if (raf) cancelAnimationFrame(raf);
      raf = 0;
      running = false;
    }

    /* All sections from the first opaque section through the footer are contiguous. */
    function updateCoverage(y) {
      var full = y >= flushTop && opaqueStart.getBoundingClientRect().top <= 0;
      if (full === covered) return;
      covered = full;
      if (covered) stop(); else kick();
    }

    function setHighlights(labels) {
      highlighted = Object.create(null);
      labels.forEach(function (label) { highlighted[label] = true; });
      applyGlow();
    }

    function selectStage(y) {
      var marker = y + H * .48, next = 0;
      while (next + 1 < stages.length && stages[next + 1].top <= marker) next++;
      if (marker >= flushTop) {
        if (stageIndex !== -2) { stageIndex = -2; setHighlights([]); }
      } else if (next !== stageIndex) {
        stageIndex = next;
        targetFill = Math.max(targetFill, stages[next].fill);
        setHighlights(stages[next].labels);
      }
      return stages[next];
    }

    function cacheStory() {
      var y = window.scrollY;
      stages = Array.from(document.querySelectorAll('[data-fill]'), function (el) {
        return {
          top: el.getBoundingClientRect().top + y,
          fill: Number(el.dataset.fill),
          slab: el.hasAttribute('data-slab'),
          labels: Array.from(el.querySelectorAll('[data-labels]')).flatMap(function (mark) {
            return mark.dataset.labels.split(',').map(function (label) { return label.trim(); });
          })
        };
      });
      flushTop = document.getElementById('flush').getBoundingClientRect().top + y;
      stageIndex = -1;
      selectStage(y);
      updateCoverage(y);
    }

    function resize() {
      var w = window.innerWidth, h = window.innerHeight, dpr = Math.min(2, window.devicePixelRatio || 1);
      if (w === W && h === H && dpr === DPR) return;
      W = w; H = h; DPR = dpr;
      capH = Math.round(H * (W < 640 ? .28 : .34));
      bandTop = H - capH;
      canvas.width = Math.round(W * DPR);
      canvas.height = Math.round(H * DPR);
      canvas.style.width = W + 'px';
      canvas.style.height = H + 'px';
      ctx.setTransform(DPR, 0, 0, DPR, 0, 0);
      COL = W < 640 ? 6 : 8;
      cols = Math.ceil(W / COL);
      sprites.clear();
      if (vortex || pendingDone) {
        vortex = null;
        var finish = pendingDone;
        pendingDone = null;
        if (finish) finish();
      }
      items = getItems();
      falling = [];
      dust = [];
      items.forEach(function (it) {
        var sprite = spriteFor(it.def, it.slab);
        it.w = sprite.w * it.widthScale;
        it.h = sprite.h;
        it.sprite = sprite.node;
      });
      repack(items, false);
      cacheStory();
      draw(performance.now());
    }

    function onResize() {
      clearTimeout(resizeT);
      resizeT = setTimeout(function () { resize(); kick(); }, 160);
    }

    function onScroll() {
      var y = window.scrollY, dy = y - lastScrollY;
      lastScrollY = y;
      updateCoverage(y);
      var stage = selectStage(y);
      if (y >= flushTop && y - dy < flushTop && opts.onReachFlush) opts.onReachFlush();
      if (dy <= 0 || y >= flushTop || covered || vortex || pendingDone) return;
      spawnBudget += dy * .03;
      var allowance = Math.min(6, Math.floor(spawnBudget));
      spawnBudget %= 1;
      var reserveSlot = stage.slab && !slabPlaced;
      var added = grow(allowance, reduce, false);
      if (reserveSlot && added < allowance && items.length + falling.length === maxItems() && dropSlab()) {
        added++;
      }
      if (added) {
        onChange(getItems());
        if (reduce) draw(performance.now()); else kick();
      }
    }

    function onVisibility() {
      if (document.hidden) stop(); else kick();
    }

    function seedHero() {
      targetFill = stages[0].fill;
      setHighlights(stages[0].labels);
      grow(maxItems(), true, true);
      draw(performance.now());
      kick();
    }

    function debugItems() {
      var snapshot = items.map(function (it) { return snapshotOf(it, it.x, it.y, true); });
      falling.forEach(function (it) { snapshot.push(snapshotOf(it, it.fx == null ? it.x : it.fx, it.y, false)); });
      if (vortex) vortex.parts.forEach(function (part) {
        if (part.k < 1) snapshot.push(snapshotOf(part.it, part.x, part.y, false));
      });
      return Object.freeze(snapshot);
    }

    function snapshotOf(it, x, y, settled) {
      return Object.freeze({ label: it.label, risk: it.risk, x: x, y: y, w: it.w, h: it.h, settled: settled });
    }

    function init() {
      resize();
      seedHero();
      Object.defineProperty(window, '__pile', { value: Object.freeze({ items: debugItems }) });
      lastScrollY = window.scrollY;
      window.addEventListener('resize', onResize, { passive: true });
      window.addEventListener('scroll', onScroll, { passive: true });
      document.addEventListener('visibilitychange', onVisibility);

      document.addEventListener('click', function (e) {
        if (covered || document.hidden || vortex || !items.length || e.clientY < bandTop) return;
        if (e.target && e.target.closest && e.target.closest('a,button,input,select,textarea,[tabindex],mark,pre,.junk,.opaque,.top,.toast,.dual,footer,.sheet,.term,.copy,.again')) return;
        for (var i = items.length - 1; i >= 0; i--) {
          var it = items[i];
          if (it.risk !== 'danger') continue;
          var dx = e.clientX - (it.x + it.w / 2), dy = e.clientY - (it.y + it.h / 2);
          var c = Math.cos(-it.a), s = Math.sin(-it.a);
          var lx = dx * c - dy * s, ly = dx * s + dy * c;
          if (Math.abs(lx) <= it.w / 2 + 2 && Math.abs(ly) <= it.h / 2 + 2) {
            onDanger(it, { x: e.clientX, y: e.clientY });
            return;
          }
        }
      }, { passive: true });
    }

    init();

    return {
      setHighlights: setHighlights,
      setPreview: function (predicate) {
        preview = typeof predicate === 'function' ? predicate : null;
        applyGlow();
      },
      flush: function (predicate, origin, onDone) {
        var gone = [];
        function keep(it) {
          if (!predicate(it)) return true;
          gone.push(it);
          return false;
        }
        items = items.filter(keep);
        falling = falling.filter(keep);
        if (!gone.length) { if (onDone) onDone(); return gone; }

        spawnBudget = 0;
        dust = [];
        if (reduce) {
          items = getItems();
          falling = [];
          repack(items, false);
          onChange(getItems());
          draw(performance.now());
          if (onDone) onDone();
          return gone;
        }

        var parts = gone.map(function (it, i) {
          var x = it.fx == null ? it.x : it.fx;
          var dx = x + it.w / 2 - origin.x;
          var dy = (it.y + it.h / 2 - origin.y) / .62;
          return { it: it, x: x, y: it.y, angle: it.a, scale: 1, alpha: 1,
            distance: Math.hypot(dx, dy), theta: Math.atan2(dy, dx),
            delay: .16 * i / Math.max(1, gone.length - 1), k: 0 };
        });
        vortex = { t: 0, end: 1.6, cx: origin.x, cy: origin.y, parts: parts, travel: 1.43 };
        pendingDone = typeof onDone === 'function' ? onDone : null;
        onChange(getItems());
        kick();
        return gone;
      },
      reset: function () {
        stop();
        items = [];
        falling = [];
        dust = [];
        vortex = null;
        pendingDone = null;
        glowUntil = 0;
        preview = null;
        spawnBudget = 0;
        slabPlaced = false;
        stageIndex = -1;
        bag = [];
        pickSeq = 0;
        heights = new Float32Array(cols);
        seedHero();
        onChange(getItems());
      },
      getItems: getItems
    };
  }


  var CATALOG = [
    /* safe: the tool rebuilds these on demand */
    { label: 'npx',              risk: 'safe',    gb: 2.3, cliId: 'npx' },
    { label: 'uvx',              risk: 'safe',    gb: 3.4, cliId: 'uv' },
    { label: 'uv cache',         risk: 'safe',    gb: 14.2, cliId: 'uv' },
    { label: 'pip wheels',       risk: 'safe',    gb: 1.8, cliId: 'pip' },
    { label: 'pnpm-store',       risk: 'safe',    gb: 6.7, cliId: 'pnpm' },
    { label: 'go-build',         risk: 'safe',    gb: 4.5, cliId: 'go' },
    { label: 'DerivedData',      risk: 'safe',    gb: 3.1, cliId: 'xcode' },
    { label: 'debug.txt',        risk: 'safe',    gb: 0.4, cliId: 'claude_debug' },
    { label: 'build cache',      risk: 'safe',    gb: 3.2, cliId: 'dk_build' },
    { label: 'npm cache',        risk: 'safe',    gb: 4.1, cliId: 'npm' },
    { label: 'Yarn cache',       risk: 'safe',    gb: 1.2, cliId: 'yarn' },
    { label: 'bun cache',        risk: 'safe',    gb: 0.9, cliId: 'bun' },
    { label: 'Electron',         risk: 'safe',    gb: 2.1, cliId: 'electron' },
    { label: 'node-gyp',         risk: 'safe',    gb: 0.3, cliId: 'nodegyp' },
    { label: 'type cache',       risk: 'safe',    gb: 0.6, cliId: 'typescript' },
    { label: 'crate cache',      risk: 'safe',    gb: 2.7, cliId: 'cargo' },
    { label: 'CocoaPods',        risk: 'safe',    gb: 1.4, cliId: 'cocoapods' },
    { label: 'pre-commit',       risk: 'safe',    gb: 0.5, cliId: 'precommit' },
    { label: 'networks',         risk: 'safe',    gb: 0.1, cliId: 'dk_net' },
    { label: 'Codex logs',       risk: 'safe',    gb: 0.3, cliId: 'codex_logs' },
    { label: 'Copilot logs',     risk: 'safe',    gb: 0.2, cliId: 'copilot_logs' },
    { label: 'OpenCode logs',    risk: 'safe',    gb: 0.4, cliId: 'opencode_logs' },
    { label: 'VS Code logs',     risk: 'safe',    gb: 1.1, cliId: 'vscode' },
    { label: 'Cursor logs',      risk: 'safe',    gb: 0.8, cliId: 'cursor' },
    { label: 'Windsurf logs',    risk: 'safe',    gb: 0.3, cliId: 'windsurf' },

    /* caution: slow or large to fetch again */
    { label: 'chromium',         risk: 'caution', gb: 1.6, cliId: 'playwright' },
    { label: 'headless_shell',   risk: 'caution', gb: 1.0, cliId: 'playwright' },
    { label: 'gguf',             risk: 'caution', gb: 18.9, cliId: 'lm_studio' },
    { label: 'safetensors',      risk: 'caution', gb: 12.4, cliId: 'hf_hub' },
    { label: 'ollama blob',      risk: 'caution', gb: 9.6, cliId: 'ollama' },
    { label: 'torch hub',        risk: 'caution', gb: 2.8, cliId: 'torch_hub' },
    { label: '.gradle',          risk: 'caution', gb: 4.2, cliId: 'gradle' },
    { label: 'Homebrew',         risk: 'caution', gb: 4.2, cliId: 'brew' },
    { label: 'Playwright',       risk: 'caution', gb: 1.6, cliId: 'playwright' },
    { label: 'Puppeteer',        risk: 'caution', gb: 1.0, cliId: 'puppeteer' },
    { label: 'hf hub',           risk: 'caution', gb: 5.3, cliId: 'hf_hub' },
    { label: 'LM Studio',        risk: 'caution', gb: 7.8, cliId: 'lm_studio' },

    /* danger: never in a preset, never previewed, typed yes only */
    { label: 'Docker volume',    risk: 'danger',  gb: 8.4, cliId: 'dk_volumes' },
    { label: 'container',        risk: 'danger',  gb: 6.2, cliId: 'dk_stopped' },
    { label: 'image layer',      risk: 'danger',  gb: 14.6, cliId: 'dk_images' },
    { label: 'rollout.jsonl',    risk: 'danger',  gb: 1.1, cliId: 'codex_history' },
    { label: 'unused images',    risk: 'danger',  gb: 5.7, cliId: 'dk_images' },
    { label: 'Claude transcripts', risk: 'danger', gb: 2.4, cliId: 'claude_history' },
    { label: 'Gemini transcripts', risk: 'danger', gb: 1.9, cliId: 'gemini_history' },

    /* stale: opt in only, never in a preset unless the checkbox is on */
    { label: 'node_modules',     risk: 'stale',   gb: 9.8, cliId: 'stale' },
    { label: '.venv',            risk: 'stale',   gb: 4.8, cliId: 'stale' },
    { label: 'target',           risk: 'stale',   gb: 3.6, cliId: 'stale' },
    { label: '.next',            risk: 'stale',   gb: 2.2, cliId: 'stale' },
    { label: 'dist',             risk: 'stale',   gb: 1.7, cliId: 'stale' },
    { label: '.turbo',           risk: 'stale',   gb: 1.3, cliId: 'stale' }
  ];

  var SLAB = { label: '~/.cache/uv', risk: 'safe', gb: 57, cliId: 'uv' };

  function createFlushControls(options) {
    var opts = options;
    var pile = opts.pile;
    var setUsed = opts.setUsed;
    var showToast = opts.showToast;

    var term = document.getElementById('term');
    var log = document.getElementById('term-log');
    var summary = document.getElementById('term-summary');
    var singleColumn = window.matchMedia('(max-width:900px)');
    var againBtn = document.getElementById('again');
    var buttons = Array.prototype.slice.call(document.querySelectorAll('.dual button[data-preset]'));
    var staleEl = document.getElementById('include-stale');
    var dangerList = document.getElementById('danger-items');
    var dangerConfirm = document.getElementById('danger-confirm');

    var reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    var busy = false;
    var active = null;
    var activePoint, returnFocus;
    var lastMeter = NaN;
    var lastDangerKey = null;

    var dangerPage = 0;

    function liveItems() { return pile.getItems(); }

    function usedFrom(items) { return SAMPLE_BASE + sumTenths(items); }

    function fmt(tenths) { return (tenths / 10).toFixed(1) + ' GB'; }

    function pad(s, n) { while (s.length < n) s += ' '; return s; }
    function labelCols() { return window.matchMedia('(max-width: 640px)').matches ? 33 : 42; }
    function fit(s, n) { return s.length > n ? s.slice(0, n - 1) + '…' : s; }
    function esc(s) { return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;'); }

    function centerOf(el) {
      if (!el) return { x: 0, y: 0 };
      var r = el.getBoundingClientRect();
      return { x: r.left + r.width / 2, y: r.top + r.height / 2 };
    }

    /* Presets. Danger is excluded from both, and never previewed. */
    function safePred(it) { return !!it && it.risk === 'safe'; }
    function bigPred(it) {
      if (!it) return false;
      if (it.risk === 'safe' || it.risk === 'caution') return true;
      return !!(staleEl && staleEl.checked && it.risk === 'stale');
    }
    function predFor(b) { return b && b.getAttribute('data-preset') === 'safe' ? safePred : bigPred; }

    function setPreview(fn) {
      pile.setPreview(fn);
    }

    // Hover and focus both keep the dry run active.
    var hovered, focused;
    function updatePreview() {
      var button = hovered || focused;
      setPreview(!busy && button ? predFor(button) : null);
    }
    buttons.forEach(function (button) {
      button.addEventListener('pointerenter', function () { hovered = button; updatePreview(); }, { passive: true });
      button.addEventListener('pointerleave', function () { hovered = null; updatePreview(); }, { passive: true });
      button.addEventListener('focus', function () { focused = button; updatePreview(); });
      button.addEventListener('blur', function () { focused = null; updatePreview(); });
    });

    /* ---------- danger list, rendered from the real live items ---------- */
    function dangerItems(items) {
      return (items || []).filter(function (it) { return it && it.risk === 'danger' && it.label; });
    }

    function renderDangerList(items) {
      var ds = dangerItems(items).sort(function (a, b) { return a.id.localeCompare(b.id); });
      var key = ds.map(function (it) { return it.id + ':' + it.tenths; }).join('|');
      if (key === lastDangerKey) return;
      lastDangerKey = key;

      var byLabel = new Map();
      ds.forEach(function (it) {
        if (!byLabel.has(it.label)) byLabel.set(it.label, []);
        byLabel.get(it.label).push(it);
      });
      var groups = Array.from(byLabel.values());
      var pageCount = Math.ceil(groups.length / 4);
      dangerPage = Math.min(dangerPage, Math.max(0, pageCount - 1));

      function paint() {
        dangerList.replaceChildren();
        var visible = groups.slice(dangerPage * 4, dangerPage * 4 + 4);
        visible.forEach(function (group) {
          var it = group[0], amount = fmt(sumTenths(group));
          var count = group.length > 1 ? ' x' + group.length : '';
          var b = document.createElement('button');
          b.type = 'button';
          b.className = 'danger-item';
          b.dataset.label = it.label;
          b.setAttribute('aria-label', it.label + count + ', ' + amount + '. Flush one item, needs yes.');
          b.title = 'Flush one ' + it.label + ' (needs yes)';
          b.innerHTML =
            '<span class="danger-item__tag" aria-hidden="true">!</span>' +
            '<span class="danger-item__label">' + esc(it.label) + '</span>' +
            (count ? '<span class="danger-item__count">' + count.trim() + '</span>' : '') +
            '<span class="danger-item__gb">' + amount + '</span>';
          b.addEventListener('click', function () { showDangerForm(it, centerOf(b), b); });
          dangerList.appendChild(b);
        });
        if (pageCount < 2) return;
        var more = document.createElement('button');
        more.type = 'button';
        more.className = 'danger-more';
        more.textContent = '+' + (groups.length - visible.length) + ' more';
        more.setAttribute('aria-label', 'Show next danger groups');
        more.addEventListener('click', function () {
          dangerPage = (dangerPage + 1) % pageCount;
          paint();
          dangerList.querySelector('.danger-more').focus({ preventScroll: true });
        });
        dangerList.appendChild(more);
      }
      paint();
    }

    /* ---------- tiny inline confirm form ---------- */
    function clearForm(restoreFocus) {
      active = null;
      dangerConfirm.textContent = '';
      delete dangerConfirm.dataset.pile;
      if (restoreFocus && returnFocus) returnFocus.focus({ preventScroll: true });
      returnFocus = null;
    }

    function placeDangerPrompt() {
      var viewport = window.visualViewport;
      if (!viewport || !dangerConfirm.hasAttribute('data-pile')) return;
      var inset = Math.max(0, innerHeight - viewport.height - viewport.offsetTop);
      dangerConfirm.style.setProperty('--keyboard-inset', inset + 'px');
    }
    window.visualViewport?.addEventListener('resize', placeDangerPrompt, { passive: true });
    window.visualViewport?.addEventListener('scroll', placeDangerPrompt, { passive: true });

    function showDangerForm(item, point, trigger) {
      if (busy) return;
      active = item;
      activePoint = point;
      returnFocus = trigger;
      dangerConfirm.toggleAttribute('data-pile', !trigger);
      placeDangerPrompt();
      dangerConfirm.textContent = '';

      var form = document.createElement('form');
      form.className = 'danger-form';
      form.setAttribute('aria-label', 'Confirm flushing ' + item.label);
      form.innerHTML =
        '<span class="danger-form__q">Flush <b>' + esc(item.label) + '</b> (' + esc(fmt(item.tenths)) +
          ')? Type <code>yes</code> in full.</span>' +
        '<input class="danger-form__input" type="text" autocomplete="off" autocapitalize="off" ' +
          'spellcheck="false" aria-label="Type yes to confirm, lower case">' +
        '<button class="danger-form__yes" type="submit">flush</button>' +
        '<button class="danger-form__no" type="button">cancel</button>' +
        '<span class="danger-form__err" role="alert" hidden>Type yes exactly, lower case.</span>';
      dangerConfirm.appendChild(form);

      var input = form.querySelector('.danger-form__input');
      var err = form.querySelector('.danger-form__err');
      var cancel = form.querySelector('.danger-form__no');

      form.addEventListener('submit', function (e) {
        e.preventDefault();
        if (!input || input.value !== 'yes') {
          if (err) err.hidden = false;
          input.focus({ preventScroll: !trigger });
          return;
        }
        var it = active;
        var p = activePoint;
        clearForm();
        if (!it) return;
        runFlush(function (x) { return !!x && x.id === it.id; }, p, 'danger', it);
      });

      cancel.addEventListener('click', function () { clearForm(true); });
      form.addEventListener('keydown', function (e) {
        if (e.key === 'Escape' || e.key === 'Esc') { e.preventDefault(); clearForm(true); }
      });

      input.focus({ preventScroll: !trigger });
    }

    /* ---------- terminal transcript, grouped by registered CLI item ---------- */
    function printRun(kind, removed, before, after, onlyItem) {
      var isDanger = kind === 'danger';
      var grouped = new Map();
      removed.forEach(function (item) {
        var id = item.def.cliId;
        grouped.set(id, (grouped.get(id) || 0) + item.tenths);
      });

      var command = 'cleancodeshit --select safe --yes';
      if (kind === 'all') {
        command = grouped.has('stale') ? 'cleancodeshit --in ~/code' : 'cleancodeshit --select all --yes';
      } else if (isDanger) {
        command = 'cleancodeshit --select ' + onlyItem.def.cliId;
      }

      var lines = ['<span class="p">$</span> ' + esc(command)];
      if (isDanger) lines.push('<span class="p">?</span> Type "yes" to continue: yes');
      lines.push('<span class="mute">  ' + grouped.size + ' item(s) cleaned</span>');
      Object.keys(CLI_LABELS).forEach(function (id) {
        if (!grouped.has(id)) return;
        var cls = isDanger ? 'red hz' : 'ok';
        var mark = isDanger ? '!' : '\u2714';
        lines.push('  <span class="term__item" data-cli-id="' + id + '"><span class="' + cls + '">' + mark + '</span> ' +
          '<span class="term__label">' + esc(pad(fit(CLI_LABELS[id], labelCols()), labelCols())) + '</span> ' +
          '<span class="term__size">' + fmt(grouped.get(id)) + '</span></span>');
      });

      log.innerHTML = lines.join('\n');
      log.scrollTop = 0;
      log.scrollLeft = 0;
      summary.innerHTML = '<span class="tot">Freed: ' + fmt(sumTenths(removed)) + '</span>' +
        '<span>Disk free: ' + fmt(SAMPLE_CAPACITY - before) + ' &gt; ' + fmt(SAMPLE_CAPACITY - after) + '</span>';
      summary.hidden = false;
    }

    function revealResult() {
      if (singleColumn.matches) {
        if (!reduce) term.scrollIntoView({ behavior: 'smooth', block: 'center' });
        return;
      }
      // Keep the result clear even if a control was pressed low in the viewport.
      var capLine = innerHeight - Math.round(innerHeight * .34);
      var bottom = term.getBoundingClientRect().bottom;
      if (bottom <= capLine - 12) return;
      term.style.setProperty('--term-lift', (capLine - 12 - bottom) + 'px');
      window.addEventListener('scroll', function () {
        term.style.removeProperty('--term-lift');
      }, { once: true, passive: true });
    }

    /* ---------- one run: safe preset, big preset, or one danger item ---------- */
    function runFlush(predicate, point, kind, onlyItem) {
      if (busy) return;
      busy = true;
      if (active) clearForm(false);
      setPreview(null);
      buttons.forEach(function (b) { b.disabled = true; });
      showToast(false);
      term.style.removeProperty('--term-lift');

      var before = usedFrom(liveItems());
      var removed = pile.flush(predicate, point, function () {
        busy = false;
        buttons.forEach(function (b) { b.disabled = false; });
        againBtn.hidden = false;
        queueMicrotask(revealResult);
      });

      var after = usedFrom(liveItems());
      printRun(kind, removed, before, after, onlyItem);
      syncItems(liveItems());
    }

    function syncItems(items) {
      renderDangerList(items);
      var v = usedFrom(items);
      if (v !== lastMeter) {
        lastMeter = v;
        setUsed(v);
      }
    }

    /* ---------- handlers handed to the engine ---------- */
    function onChange(items) { syncItems(items || liveItems()); }

    function onDanger(item, point) { showDangerForm(item, point); }

    /* ---------- buttons, make-it-again, stale toggle ---------- */
    buttons.forEach(function (b) {
      b.addEventListener('click', function () {
        if (busy) return;
        var preset = b.getAttribute('data-preset');
        var kind = preset === 'safe' ? 'safe' : 'all';
        b.classList.add('pressed');
        setTimeout(function () { b.classList.remove('pressed'); }, 260);
        runFlush(kind === 'safe' ? safePred : bigPred, centerOf(b), kind, null);
      });
    });

    staleEl.addEventListener('change', updatePreview);
    againBtn.addEventListener('click', function () {
      againBtn.hidden = true;
      clearForm(false);
      staleEl.checked = false;
      pile.reset();
      log.innerHTML = '<span class="mute"># Scroll down to build the pile again.</span>';
      summary.replaceChildren();
      summary.hidden = true;
      term.style.removeProperty('--term-lift');
      window.scrollTo({ top: 0, behavior: 'instant' });
    });

    /* First paint of the danger list and the meter from real live items. */
    syncItems(liveItems());

    return { onChange: onChange, onDanger: onDanger };
  }

  /* highlights on touch */
  document.addEventListener('click', function (e) {
    var m = e.target.closest('.junk');
    document.querySelectorAll('.junk.open').forEach(function (o) { if (o !== m) o.classList.remove('open'); });
    if (m) m.classList.toggle('open');
  });

  /* ---------- toilet roll unroll ---------- */
  var sheets = document.querySelectorAll('.sheet');
  var sio = new IntersectionObserver(function (entries) {
    entries.forEach(function (e) {
      if (!e.isIntersecting) return;
      sheets.forEach(function (s, i) { setTimeout(function () { s.classList.add('in'); }, reduce ? 0 : i * 260); });
      sio.disconnect();
    });
  }, { threshold: 0.2 });
  sio.observe(document.querySelector('.sheets'));

  /* copy buttons */
  document.querySelectorAll('[data-copy]').forEach(function (b) {
    b.addEventListener('click', function () {
      var text = document.getElementById(b.getAttribute('data-copy')).innerText;
      function ok() { b.textContent = 'Copied'; setTimeout(function () { b.textContent = 'Copy'; }, 1400); }
      function fallback() {
        var ta = document.createElement('textarea'); ta.value = text; document.body.appendChild(ta); ta.select();
        try { document.execCommand('copy'); ok(); } catch (err) { /* ignore */ }
        ta.remove();
      }
      if (navigator.clipboard && window.isSecureContext) navigator.clipboard.writeText(text).then(ok, fallback);
      else fallback();
    });
  });


  document.fonts.ready.then(function () {
    var controls;
    var pile = createPile({
      canvas: document.getElementById('pile'),
      reduce: reduce,
      catalog: CATALOG,
      slab: SLAB,
      onChange: function (items) { if (controls) controls.onChange(items); },
      onDanger: function (item, point) { controls.onDanger(item, point); },
      onSlabLand: function () { showToast(true); },
      onReachFlush: function () { showToast(false); }
    });
    controls = createFlushControls({ pile: pile, setUsed: setUsed, showToast: showToast });
  });
})();
