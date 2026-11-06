MOMENTS.push({ name: 'Shooting star', from: 'Genshin Impact wishes', look: 'celestial', spin: 'drop',
    d: 'The page sinks into a night sky with drifting nebulae and dark hills. Your stars join into a constellation in the shape of the Inventory gem, one star breaks off and streaks across the sky, changing colour in flight. It crashes in a white-out that lights the hills, and the card spins out of the light as a real 3D card.',
    async play(id) {
      const C = RC[rarity], steps = TEASE[rarity];
      await shareTap(id);
      A(shop, [{ filter: 'none', transform: 'none' }, { filter: 'blur(6px) brightness(.35)', transform: 'translateY(120px) scale(.92)' }], 700, { ease: 'cubic-bezier(.5,0,.3,1)' });
      const sky = backdrop(`linear-gradient(180deg, #020A0F 0%, #06181D 40%, #0D2E34 75%, #1E4E55 100%)`);
      const neb = add(`<div style="position:absolute;inset:-12%;background:radial-gradient(38% 26% at 28% 28%, rgba(63,122,130,.45), transparent 70%), radial-gradient(30% 22% at 74% 20%, rgba(110,80,130,.32), transparent 70%), radial-gradient(46% 26% at 60% 52%, rgba(30,78,85,.4), transparent 70%), radial-gradient(20% 12% at 40% 46%, rgba(167,211,204,.12), transparent 70%);opacity:0"></div>`);
      A(neb, [{ opacity: 0 }, { opacity: 1 }], 900); A(neb, [{ transform: 'translate(0,0) scale(1)' }, { transform: 'translate(-14px,8px) scale(1.06)' }], 9000, { ease: 'ease-in-out' });
      const hills = add(`<svg width="360" height="230" viewBox="0 0 360 230" style="position:absolute;left:0;bottom:0;opacity:0">
        <path d="M0 110 C40 92 70 104 104 86 C140 66 172 96 212 80 C252 64 292 90 360 70 V230 H0Z" fill="#0A2226"/><path class="rim" d="M0 110 C40 92 70 104 104 86 C140 66 172 96 212 80 C252 64 292 90 360 70" fill="none" stroke="${C}" stroke-width="1.5" opacity="0"/>
        <path d="M0 150 C50 128 90 146 140 124 C190 104 230 136 280 120 C320 108 340 118 360 112 V230 H0Z" fill="#071A1D"/><path class="rim" d="M0 150 C50 128 90 146 140 124 C190 104 230 136 280 120 C320 108 340 118 360 112" fill="none" stroke="${C}" stroke-width="1.5" opacity="0"/>
        <path d="M0 192 C60 170 110 186 170 168 C230 150 280 178 360 160 V230 H0Z" fill="#040E10"/><path class="rim" d="M0 192 C60 170 110 186 170 168 C230 150 280 178 360 160" fill="none" stroke="${C}" stroke-width="2" opacity="0"/></svg>`);
      A(hills, [{ opacity: 0, transform: 'translateY(40px)' }, { opacity: 1, transform: 'none' }], 800);
      const hz = light({ kind: 'glow', x: 180, y: 800, r: 300, color: '#2D6A72', a: 0, sy: .5 }); tw(hz, { a: .6 }, 600);
      const stars = emit(() => glints(rnd(10, 350), rnd(30, 480), '#A7D3CC', 1, 1, { life: 2.6, size: 1.2 }), 18, 60);
      for (let i = 0; i < 26; i++) glints(rnd(10, 350), rnd(30, 480), '#A7D3CC', 1, 1, { life: 2.4, size: 1.1, delay: .6 });
      camTo(1.02, 600, -10);
      await wait(420, id);
      // A constellation in the shape of the gem draws itself.
      const pts0 = [[180, 70], [222, 104], [180, 176], [138, 104], [160, 88], [200, 88]];
      const con = add(`<svg width="360" height="300" style="position:absolute;left:0;top:0;overflow:visible"><path d="M180 70 L222 104 L180 176 L138 104 Z M138 104 H222 M160 88 L180 104 L200 88" fill="none" stroke="#A7D3CC" stroke-opacity=".55" stroke-width="1" stroke-dasharray="420" stroke-dashoffset="420"/>${pts0.map(([x, y]) => `<circle cx="${x}" cy="${y}" r="2" fill="#F4F2EF"/>`).join('')}</svg>`);
      A(con.querySelector('path'), [{ strokeDashoffset: 420 }, { strokeDashoffset: 0 }], 700, { ease: 'ease-in-out' });
      pts0.forEach(([x, y], i) => glints(x, y, '#A7D3CC', 1, 1, { life: 1.4, size: 1.8, delay: i * .08 }));
      SFX.play('shimmer');
      await wait(760, id);
      const c = light({ kind: 'comet', x: 180, y: 70, color: steps[0], w: 3.4, trail: [], n: 30 });
      sparks(180, 70, '#FFFFFF', 16, 160, { g: 0 }); ring(180, 70, '#FFFFFF', 40, 2, .4); SFX.play('ting', 0);
      out(con, 600);
      const path = { u: 0 }, P0 = [180, 70], P1 = [470, 120], P2 = [180, 362];
      const at = u => [(1 - u) * (1 - u) * P0[0] + 2 * (1 - u) * u * P1[0] + u * u * P2[0], (1 - u) * (1 - u) * P0[1] + 2 * (1 - u) * u * P1[1] + u * u * P2[1]];
      const shed = emit(() => { [c.x, c.y] = at(path.u); sparks(c.x, c.y, c.color, 2, 90, { g: 60, life: .7, drag: 1.5, white: .5 }); if (Math.random() < .3) glints(c.x, c.y, c.color, 1, 6, { life: .6 }); }, 120, 10);
      const flight = 1400 + 250 * (steps.length - 1);
      SFX.play('meteor', flight / 1000 + .2); SFX.play('riser', flight / 1000);
      const go = tw(path, { u: 1 }, flight, p => .55 * p + .45 * p * p);
      const marks = steps.length === 3 ? [.38, .68] : steps.length === 2 ? [.55] : [];
      for (let i = 0; i < marks.length; i++) {
        while (path.u < marks[i]) { await wait(16, id); }
        const col = steps[i + 1]; c.color = col; c.w += 1;
        const [x, y] = at(path.u); SFX.play('ting', i + 1); kick(.12 + .08 * i);
        light({ kind: 'flare', x, y, color: col, len: 170 + 60 * i, th: 6, core: 70, a: 1, life: .55 }); ring(x, y, col, 70 + 30 * i, 4, .5); sparks(x, y, col, 24, 260, { g: 100 });
        light({ kind: 'bloom', x, y, color: col, a: .4, R: 260, life: .5 });
      }
      while (path.u < .9) { await wait(16, id); }
      SFX.play('suck', .25);
      await go; guard(id);
      shed.dead = true; c.dead = true;
      envFor(rarity); glOn();
      const card = hiddenCard(180, 362, { sc: .55 }); flash(.6, .45); ring(180, 362, '#FFFFFF', 260, 5, .5);
      const rv = reveal3(card, { y: 362, z: .3, onFace: () => { flash(.3, .35); hills.querySelectorAll('.rim').forEach((p, i) => A(p, [{ opacity: 0 }, { opacity: 1, offset: .15 }, { opacity: .35 }], 1600, { delay: i * 60 })); } });
      camTo(1, 900, 0, 'cubic-bezier(.2,.8,.2,1)');
      await rv; guard(id);
      await settle(id, card, { delay: 400, pointsY: 526 });
      stars.dead = true;
    } });
