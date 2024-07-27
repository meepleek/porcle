# Todo

## Overview

- circle pong
- reflecting the ball adds the current mult to points
- circling the paddle around raises the mult
- points can be used for upgrades
- possibly add a timer
- upgrading slows ball, but also resets mult
- upgrades initially show only the price, but the upgrade itself are just ???


## Upgrades

- shield
- bigger paddle
- add another paddle
- raise starting mult
- add time
- slowmo meter
- add extra scoring ball (different color?) that scores but doesn't lose the game

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
- [ ] bullet accurracy (couple degrees)
- [ ] ammo capacity
- [ ] ammo UI
- [ ] health UI
- [ ] circle/anullus transition
- [ ] tween in lvl elements
- [ ] spawn initial ball after a delay
- [ ] add reflection/aim prediction UI
- [ ] score - just kill count

## audio - v0.6.0

## enemies - v0.7.0

- [ ] improve crawler
- [ ] slower crawler that has to be hit by the ball (shielded)
- [ ] shooty enemy/turret
- [ ] turret that has to be destroyed by the ball (shielded)
- [ ] paddle blocks enemy projectiles
- [ ] ball destroys enemy projectiles

## upgrades - v0.x.0

- [ ] health
- [ ] max health
- [ ] more homing
- [ ] ball grows bigger
- [ ] faster shooting
- [ ] bigger ammo reflection
- [ ] greater ammo capacity

## extra VFX - v0.7.0

- [ ] add vignette?
- [ ] add chromatic abberation

## scrapped

- [ ] enemy death shockwave
- [ ] reflect shockwave
- [ ] ball trail
- [ ] paddle trail
