# Core Fighting Loop

This document defines the intended model for future Stellar Blade combat tuning. It is design intent, not discovered runtime fact. Concrete object paths, functions, offsets, and hook points must come from reconnaissance output before implementation.

## Player Combat States

- `Neutral`: player can move, attack, guard, dodge, or enter contextual actions.
- `AttackStartup`: player attack committed but not yet active; defensive cancels depend on discovered game rules.
- `AttackActive`: player hitbox or damage application is active.
- `AttackRecovery`: player cannot freely act until recovery or cancel windows open.
- `GuardReady`: player is eligible for block or parry evaluation.
- `ParryWindow`: timed interval where an incoming enemy strike can become a parry.
- `PerfectParryWindow`: stricter timing interval for stronger reward, if supported by the game.
- `DodgeStartup`: dodge input accepted but invulnerability or displacement may not be active yet.
- `DodgeInvulnerable`: enemy attacks should miss unless the game marks them as unavoidable.
- `DodgeRecovery`: player has limited action choices after evade movement.
- `Hitstun`: player is reacting to damage or guard break.
- `PunishReady`: player has earned a counterattack or special response opportunity.

## Enemy Combatant States

- `IdleOrPatrol`: not actively engaged.
- `AcquireTarget`: selecting the player or another combat target.
- `Telegraph`: attack intent is visible; parry/dodge timing can be computed from this phase.
- `AttackStartup`: strike is committed.
- `AttackActive`: strike can collide or apply damage.
- `AttackRecovery`: enemy is vulnerable to punish actions.
- `Staggered`: balance or posture is broken.
- `HitReaction`: enemy is responding to player damage.
- `Cooldown`: enemy cannot immediately repeat selected actions.

## Resolution Order

1. Read current player state, enemy intent, active enemy attacks, and frame delta.
2. Evaluate player defensive input windows before damage resolution.
3. Resolve parry before dodge when both are eligible in the same frame, unless discovery proves Stellar Blade uses another priority.
4. If parry succeeds, suppress incoming damage, trigger parry reaction, and open any punish or counter window.
5. If parry fails but dodge invulnerability is active, suppress or avoid the incoming attack according to discovered attack metadata.
6. If neither defense succeeds, resolve guard, damage, hit reaction, stagger, or death according to discovered game data.
7. Update cooldowns, recovery timers, and enemy next-intent constraints.

## Future Preset Categories

- `ParryTiming`: startup frames, active frames, perfect-parry frames, recovery, chain-parry behavior.
- `DodgeTiming`: startup, invulnerability, recovery, cancel windows, directional restrictions.
- `EnemyIntent`: telegraph duration, attack commitment, punish vulnerability, cooldown, aggression budget.
- `DamageResponse`: hitstun, guard damage, stagger/balance changes, counterattack reward.
- `DifficultyScaling`: per-difficulty multipliers for timing windows, enemy recovery, and resource rewards.

## Failure Cases

- Missing discovery API: the runtime must log that a V2 shim is required.
- Ambiguous candidates: keep multiple records; do not choose a hook target automatically.
- Conflicting timing evidence: prefer captured runtime traces over names or inferred semantics.
- Online or anti-cheat context: do not patch or attach.
