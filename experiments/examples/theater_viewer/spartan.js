/* Procedural presentation models, not extracted game assets or decoded joint poses.
   All articulation is a pure function of the film playhead and accepted observations. */
(() => {
  'use strict';
  function create(THREE, body, scale, color) {
    const root = new THREE.Group(); root.scale.setScalar(scale); body.add(root);
    const mat = (color, extra = {}) => new THREE.MeshStandardMaterial({ color, roughness: .6, metalness: .35, transparent: true, ...extra });
    const armor = mat(color), highlight = mat(new THREE.Color(color).lerp(new THREE.Color('#d8e4ef'), .22));
    const suit = mat('#20272c', { roughness: .92, metalness: .05 }), joint = mat('#444c51');
    const visor = mat('#cc9a46', { roughness: .16, metalness: .78, emissive: '#4e2c0e', emissiveIntensity: .3 });
    const gunMetal = mat('#343b3c'), gunPanel = mat('#707a69'), barrel = mat('#171e23', { metalness: .7 });
    const lens = mat('#559caf', { roughness: .15, emissive: '#1a555e', emissiveIntensity: .45 });
    const materials = [armor, highlight, suit, joint, visor, gunMetal, gunPanel, barrel, lens];
    const add = (geometry, material, parent, x = 0, y = 0, z = 0) => {
      const mesh = new THREE.Mesh(geometry, material); mesh.position.set(x, y, z); parent.add(mesh); return mesh;
    };
    const box = (parent, material, x, y, z, w, h, d) => add(new THREE.BoxGeometry(w, h, d), material, parent, x, y, z);
    function plate(parent, material, x, y, z, w, h, d, cut = .055) {
      const c = Math.min(cut, w / 4, h / 4), shape = new THREE.Shape();
      shape.moveTo(-w / 2 + c, -h / 2);
      for (const [px, py] of [[w / 2 - c, -h / 2], [w / 2, -h / 2 + c], [w / 2, h / 2 - c], [w / 2 - c, h / 2], [-w / 2 + c, h / 2], [-w / 2, h / 2 - c], [-w / 2, -h / 2 + c]]) shape.lineTo(px, py);
      shape.closePath();
      const bevel = Math.min(.012, d / 5);
      const geometry = new THREE.ExtrudeGeometry(shape, { depth: d - bevel * 2, steps: 1, bevelEnabled: true, bevelSegments: 1, bevelSize: bevel, bevelThickness: bevel });
      geometry.translate(0, 0, -d / 2 + bevel);
      return add(geometry, material, parent, x, y, z);
    }
    const sphere = (parent, material, x, y, z, radius) => add(new THREE.SphereGeometry(radius, 10, 8), material, parent, x, y, z);
    function tube(parent, material, x, y, z, radius, length) {
      const m = add(new THREE.CylinderGeometry(radius, radius, length, 12), material, parent, x, y, z); m.rotation.x = Math.PI / 2; return m;
    }
    const spine = new THREE.Group(); root.add(spine);
    plate(spine, suit, 0, 1.34, 0, .67, .69, .35);
    plate(spine, armor, 0, 1.46, .08, .86, .6, .43);
    for (const sign of [-1, 1]) {
      const chest = plate(spine, highlight, sign * .21, 1.56, .33, .37, .28, .1); chest.rotation.z = sign * -.1;
      plate(spine, armor, sign * .32, 1.22, .14, .2, .26, .25);
      plate(spine, gunMetal, sign * .23, 1.47, -.29, .25, .51, .18);
      for (let i = 0; i < 3; i++) box(spine, joint, sign * .23, 1.48 + i * .075, -.39, .19, .025, .02);
    }
    plate(spine, armor, 0, 1.4, -.28, .26, .37, .16);
    box(spine, joint, 0, 1.56, .395, .065, .19, .015);
    for (let i = 0; i < 3; i++) plate(spine, joint, 0, 1.13 - i * .075, .15, .43, .05, .09, .015);
    plate(root, suit, 0, .83, 0, .57, .29, .32);
    plate(root, armor, 0, .86, .18, .54, .22, .13);
    for (const sign of [-1, 1]) plate(root, armor, sign * .34, .86, 0, .14, .3, .35);

    const head = new THREE.Group(); head.position.y = 1.96; spine.add(head);
    sphere(spine, suit, 0, 1.78, 0, .12);
    plate(head, armor, 0, .015, 0, .55, .47, .49, .1);
    plate(head, highlight, 0, .235, -.015, .34, .09, .35, .035);
    plate(head, visor, 0, .045, .257, .47, .18, .047, .055);
    plate(head, armor, 0, .158, .275, .52, .085, .065, .025);
    plate(head, suit, 0, -.13, .255, .31, .12, .05, .02);
    for (const sign of [-1, 1]) {
      plate(head, armor, sign * .22, -.09, .23, .12, .23, .14, .025);
      plate(head, joint, sign * .285, .02, -.025, .045, .19, .22, .02);
    }

    const legs = [-1, 1].map(sign => {
      const hip = new THREE.Group(); hip.position.set(sign * .22, .89, 0); root.add(hip);
      plate(hip, suit, 0, -.22, 0, .25, .43, .24);
      plate(hip, armor, 0, -.19, .075, .28, .34, .22);
      plate(hip, highlight, sign * .125, -.18, .015, .07, .31, .24, .025);
      const knee = new THREE.Group(); knee.position.y = -.43; hip.add(knee);
      sphere(knee, joint, 0, 0, 0, .11);
      plate(knee, armor, 0, -.005, .13, .26, .21, .13, .045);
      plate(knee, suit, 0, -.22, -.01, .2, .38, .2);
      plate(knee, armor, 0, -.23, .07, .25, .34, .23, .04);
      box(knee, highlight, 0, -.22, .2, .075, .24, .028);
      const foot = new THREE.Group(); foot.position.set(0, -.38, .05); knee.add(foot);
      plate(foot, gunMetal, 0, -.015, .055, .28, .16, .42, .035);
      box(foot, suit, 0, -.09, .04, .29, .035, .44);
      return { hip, knee, foot, sign };
    });

    const arms = [-1, 1].map(sign => {
      const shoulder = new THREE.Vector3(sign * .51, 1.62, .005);
      sphere(spine, joint, shoulder.x, shoulder.y, shoulder.z, .14);
      const pauldron = plate(spine, armor, sign * .62, 1.63, 0, .31, .34, .37, .065); pauldron.rotation.z = sign * -.14;
      box(spine, highlight, sign * .66, 1.74, .02, .21, .08, .31);
      const upper = new THREE.Group(), forearm = new THREE.Group(); spine.add(upper, forearm);
      plate(upper, suit, 0, 0, 0, .2, .42, .21);
      plate(upper, armor, 0, .015, .075, .23, .32, .19);
      const elbow = sphere(spine, joint, 0, 0, 0, .105);
      plate(forearm, suit, 0, 0, 0, .17, .36, .17);
      plate(forearm, armor, 0, 0, .055, .24, .31, .23);
      const hand = plate(spine, suit, 0, 0, 0, .16, .16, .19, .025);
      return { shoulder, upper, forearm, elbow, hand, sign };
    });

    // The rifle's origin is its receiver, not its buttstock. Leave clearance for
    // the rear stock so the whole weapon sits in front of the weapon-side armor.
    const gun = new THREE.Group(); gun.position.set(-.55, 1.48, .85); gun.scale.setScalar(.85); spine.add(gun);
    function rifle(kind) {
      const g = new THREE.Group(); gun.add(g);
      const sniper = kind === 'sniper', generic = kind === 'unknown';
      const receiverLength = sniper ? .88 : .7, barrelLength = sniper ? .77 : .35;
      plate(g, gunMetal, 0, .02, .1, .22, .25, receiverLength, .035);
      plate(g, generic ? gunMetal : gunPanel, 0, .055, -.39, .22, .31, .38, .03);
      box(g, suit, 0, .035, -.595, .24, .32, .045);
      box(g, barrel, 0, .185, .06, .12, .045, receiverLength);
      tube(g, barrel, 0, .055, receiverLength / 2 + barrelLength / 2 + .07, sniper ? .038 : .045, barrelLength);
      const tip = receiverLength / 2 + barrelLength + .08;
      tube(g, gunMetal, 0, .055, tip, sniper ? .07 : .065, .12);
      tube(g, barrel, 0, .055, tip + .064, .031, .007);
      plate(g, gunMetal, 0, -.15, .36, .18, .2, .4, .025);
      const grip = plate(g, suit, 0, -.22, -.015, .12, .27, .16, .015); grip.rotation.x = -.22;
      const magazine = plate(g, generic ? gunMetal : gunPanel, 0, -.24, -.25, .15, sniper ? .28 : .34, .19, .02); magazine.rotation.x = .12;
      if (!generic) {
        const scopeZ = sniper ? .08 : .0, scopeLength = sniper ? .53 : .24;
        box(g, barrel, 0, .255, scopeZ, .09, .12, .17);
        tube(g, gunMetal, 0, .34, scopeZ, sniper ? .082 : .077, scopeLength);
        tube(g, barrel, 0, .34, scopeZ + scopeLength / 2, sniper ? .099 : .085, .065);
        tube(g, lens, 0, .34, scopeZ + scopeLength / 2 + .034, sniper ? .072 : .06, .009);
        box(g, joint, .08, .34, scopeZ, .05, .07, .09);
      }
      if (sniper) {
        for (const sign of [-1, 1]) {
          box(g, barrel, sign * .075, -.15, .58, .035, .045, .51);
          plate(g, joint, sign * .12, .03, .26, .025, .14, .35, .015);
        }
      } else {
        for (let i = 0; i < 4; i++) box(g, barrel, .115, .03, .15 + i * .075, .009, .05, .025);
      }
      return { mesh: g, muzzle: tip + .12 };
    }
    const weapons = { bandit: rifle('bandit'), sniper: rifle('sniper'), unknown: rifle('unknown') };
    const muzzle = new THREE.Group(); gun.add(muzzle);
    const flame = add(new THREE.OctahedronGeometry(.15), new THREE.MeshBasicMaterial({ color: '#ffbb59', toneMapped: false }), muzzle);
    flame.scale.set(.7, .7, 1.8);
    add(new THREE.SphereGeometry(.06, 8, 6), new THREE.MeshBasicMaterial({ color: '#fff4d8', toneMapped: false }), muzzle);
    muzzle.visible = false;
    const up = new THREE.Vector3(0, 1, 0);
    function link(part, a, b, length) {
      const direction = b.clone().sub(a);
      part.position.copy(a).add(b).multiplyScalar(.5);
      part.quaternion.setFromUnitVectors(up, direction.clone().normalize());
      part.scale.set(1, direction.length() / length, 1);
    }
    function poseArm(arm, handTarget) {
      const a = arm.shoulder, reach = handTarget.clone().sub(a), distance = reach.length();
      const hand = handTarget.clone(); reach.normalize();
      const middle = a.clone().add(hand).multiplyScalar(.5);
      const bend = new THREE.Vector3(arm.sign * .5, -.85, -.12);
      bend.addScaledVector(reach, -bend.dot(reach)).normalize();
      const elbow = middle.addScaledVector(bend, Math.sqrt(Math.max(.005, .5 ** 2 - (distance / 2) ** 2)));
      link(arm.upper, elbow, a, .42); link(arm.forearm, hand, elbow, .36);
      arm.elbow.position.copy(elbow); arm.hand.position.copy(hand); arm.hand.rotation.copy(gun.rotation);
    }
    const pulse = (active, age, duration) => active && age >= 0 && age < duration ? Math.sin(Math.PI * age / duration) : 0;
    function pose(state, time, movement = 0) {
      const name = state.weapon.name;
      const weapon = name === 'S7 Sniper' ? 'sniper' : name === 'Bandit EVO' ? 'bandit' : 'unknown';
      for (const [key, model] of Object.entries(weapons)) model.mesh.visible = key === weapon;
      muzzle.position.set(0, .055, weapons[weapon].muzzle);
      const recoil = pulse(state.firing, time - state.firingTime, .15);
      const melee = pulse(state.melee, time - state.meleeTime, .35);
      const throwing = pulse(state.grenade, time - state.grenadeTime, .45);
      const reload = pulse(state.reload, time - state.reloadTime, .5);
      const crouch = !state.dead && state.crouch.value === true ? 1 : 0;
      const stride = !state.dead && !state.stale ? Math.min(1, movement) : 0;
      const phase = time * 9 + Number(state.id) * 1.7;
      const lean = crouch * .34, gait = stride * (1 - crouch * .7);
      spine.rotation.set(lean + .12 * melee, -.28 * melee, .025 * Math.sin(phase) * gait);
      // Hinge at the waist, rather than swinging the torso around the feet.
      const waist = new THREE.Vector3(0, .95, 0);
      spine.position.copy(waist).sub(waist.clone().applyEuler(spine.rotation)).multiplyScalar(crouch);
      let lowestAnkle = Infinity;
      for (const leg of legs) {
        const walk = Math.sin(phase + (leg.sign < 0 ? Math.PI : 0));
        leg.hip.position.x = leg.sign * (.22 + crouch * .12);
        leg.hip.rotation.set(walk * .48 * gait - crouch * 1.25, 0, leg.sign * crouch * .12);
        leg.knee.rotation.x = .08 + Math.max(0, -walk) * .66 * gait + crouch * 2.42;
        leg.foot.quaternion.copy(leg.hip.quaternion).multiply(leg.knee.quaternion).invert();
        const ankle = leg.foot.position.clone().applyQuaternion(leg.knee.quaternion).add(leg.knee.position)
          .applyQuaternion(leg.hip.quaternion).add(leg.hip.position);
        lowestAnkle = Math.min(lowestAnkle, ankle.y);
      }
      // Lower the hips to the bent legs while keeping the lowest sole on the
      // same ground plane. This never changes the recorded avatar position.
      const crouchDrop = crouch * Math.max(0, lowestAnkle - .1075) * scale;
      body.position.y = -crouchDrop + (1 - crouch) * Math.abs(Math.sin(phase)) * .025 * stride * scale;
      const pitch = state.aim ? (state.aim.pitchRaw - 1024) * Math.PI * 2 / 2048 : 0;
      head.rotation.set(-pitch - lean, .05 * melee, 0);
      gun.position.set(-.55 + .18 * melee, 1.48 + crouch * .2 + .09 * melee - .09 * reload, .85 - recoil * .12 + melee * .4);
      gun.rotation.set(-pitch - lean - recoil * .09 + reload * .28, -.85 * melee, .08 * melee - .18 * reload);
      gun.updateMatrix();
      const rightGrip = new THREE.Vector3(0, -.19, -.01).applyMatrix4(gun.matrix);
      const leftGrip = new THREE.Vector3(.025, -.12, .2).applyMatrix4(gun.matrix);
      if (throwing) leftGrip.set(.55, 1.35 + throwing * .8, .3 + throwing * .65);
      else if (reload) leftGrip.lerp(new THREE.Vector3(-.12, 1.05, .15), reload);
      poseArm(arms[0], rightGrip); poseArm(arms[1], leftGrip);
      head.updateMatrix(); spine.updateMatrix();
      const eyeOffset = new THREE.Vector3(0, .045, .28).applyMatrix4(head.matrix).applyMatrix4(spine.matrix)
        .multiplyScalar(scale).applyEuler(body.rotation).add(body.position).toArray();
      return { weapon, recoil, melee, throwing, reload, crouch, stride, crouchDrop, eyeOffset };
    }
    return { root, head, gun, muzzle, materials, pose };
  }
  globalThis.TheaterSpartan = { create };
})();
