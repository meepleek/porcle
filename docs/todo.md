# Todo

## Overview

- circle pong - the core holds the ball
- reflecting the ball adds the current mult to points
- circling the paddle around adds ammo or respawns ball
- enemies come from outside & try to attack the core
- hitting the core lowers it's HP
- both the paddle gun & the ball are effective weapons
- kill count as score

## proto - v0.1.0

- [x] move paddle
- [x] move ball
- [x] reflect ball
- [x] add restart
- [x] add crawling enemies
- [x] destroy enemy on collision 
- [x] reload ball on cycle
- [x] speed up ball on reflect
- [x] charge weapon on reflect
- [x] shoot weapon (just peashooter)
- [x] destroy enemy when shot

## MVP - v0.2.0
- [x] homing ball
- [x] ball is reflected off of edges
- [x] ball slows down when it hits an edge
- [x] add damping to a fired ball
- [x] auto despawn a very slow or stationary ball
- [x] crawlers deal dmg on core collision
- [x] reflecting increases an ammo mult which gets reset on ball reset
- [x] add basic UI showing ammo & health

## MVPier - v0.3.0

- [x] reflect based on paddle hit position
- [x] limit reflection angle
- [x] never reflect behind paddle, always in front (mirror regular reflection angle)
- [x] limit paddle movement speed (angular vel)
- [x] make the core an aim dead zone
- [x] allow catching ball to aim it

## Juiiiice - v0.4.0

- [x] screen shake
- [x] paddle recoil on reflect
- [x] move gun barrel on shot
- [x] tween stuff
- [x] enemy death particles
- [x] enemy health
- [x] enemy knockback
- [x] enemy hit flash
- [x] reflect particles
- [x] reflect freeze frames
- [x] freeze frames on enemy kill
- [x] try auto-targeting closing ball enemy in cone of vision instead of homing
- [x] change ball color
- [x] add bloom
- [x] boost PP effects based on current ball speed

## Missing stuff - v0.5.0

- [x] CCW rotation adds ammo, CW twice reloads ball
- [x] randomise initial ball angle
- [x] gun accurracy (couple degrees rng)
- [x] ball reflects back from paddle hit outside
- [x] captured ball loses speed
- [x] ammo bonus based on ball speed instead of reflection count (to encourage making the ball go fast)
- [x] disallow catching ball from outside
- [x] spawn ball in captured position instead of in the center
- [x] reset capture status on ball respawn
- [x] grow ball based on speed
- [x] fix initial ball boost (probably incorrect value when no balls)
- [x] boost speed outside core?
- [x] make core actually smaller than paddle radius
- [x] fix initial ball respawn direction (incorrect after rotating)
- [x] ammo capacity
- [x] ammo UI
- [x] health UI
- [x] circle/anullus transition
- [x] tween in lvl elements
- [x] score - just kill count
- [x] add capture UI
- [x] cycle effect (reload ammo, respawn ball) juice effects - shake, particles, color
- [x] enemy death particles on core hit
- [x] bg particles
- [x] ball move particles
- [x] resize cam to fit the game
- [x] try to zoom out & make paddle radius bigger (but still zoomed out, so smaller on screen)
- [x] add black bars to sides (letterboxing)
- [x] add gamepad controls
- [x] despawn out of bounds projectiles
- [x] pick a color palette
- [x] particle color palette
- [x] UI color palette
- [x] captured ball can't destroy enemies

## enemies - v0.6.0

- [x] spawn rate based on score
- [x] improve crawler (add a sprite, too)
- [x] bigger/chunkier crawler (add a sprite, too)
- [x] slower crawler that has to be hit by the ball (shielded)

# shooty enemies - v0.7.0

- shooty enemy/turret
  - [x] enemies stop at a given distance from the core
  - [x] spawn only in corners
  - [x] enemies start shooting after reaching their stop position
  - [x] enemy projectiles dmg core
- [x] paddle blocks enemy projectiles
- [x] ball destroys enemy projectiles
- [x] when taking damage, then destroy all enemies and projectiles in paddle radius
- [ ] enemy barrel tweening
- [ ] telegraph enemy shots/shot charge/cooldown
- [ ] turret that has to be destroyed by the ball (shielded)
- [ ] make big_boi just reflect the ball and take about 5 dmg (enough to make everything else a 1 hit)

## audio - v0.8.0

## upgrades - v0.x.0

- [ ] health
- [ ] max health
- [ ] more homing
- [ ] ball grows bigger
- [ ] faster shooting
- [ ] bigger ammo reflection bonus
- [ ] greater ammo capacity
- [ ] paddle size
- [ ] better accuracy

## extra VFX - v0.7.0

- [ ] add vignette?
- [ ] add chromatic abberation
- [ ] paddle move particles
- [ ] enemy move particles

## Nice to haves

- [ ] add reflection/aim prediction UI

## scrapped

- [ ] enemy death shockwave
- [ ] reflect shockwave
- [ ] ball trail
- [ ] paddle trail
