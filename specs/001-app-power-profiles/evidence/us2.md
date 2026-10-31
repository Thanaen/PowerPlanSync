# US2 : matrice des protections (T027)

État au 31 octobre 2026, sur la machine de référence de [us1.md](us1.md). Chaque ligne reprend
le tableau « Vérifier les protections » de [quickstart.md](../quickstart.md). Une preuve
automatisée passe par un adaptateur simulé ; un essai réel change le plan Windows avec
l'exécutable témoin. Les essais réels ont été exécutés un par un et le plan initial du PC
(Haute performance) a été relu identique après chacun.

| Scénario | Preuve automatisée | Essai réel | Statut |
| --- | --- | --- | --- |
| Deux règles actives | `ordered_rules_recompute_immediately_for_explicit_edits` | `two-rules-priority` : seconde règle, première, seconde après la grâce, retour, sans plan intermédiaire | Vérifié |
| Plan changé manuellement en jeu | `revoked_record_never_restores_even_when_original_guid_returns`, `protected_exit_preserves_external_plan` | `manual-external-override` : le plan choisi reste après la fermeture du témoin | Vérifié |
| Reprendre pendant cette suspension | `resume_after_manual_override_starts_new_cycle` | — | Automatisé seulement |
| Pause puis relance | `pause_survives_restart_and_failed_return_requires_retry`, `reload_keeps_a_pause_found_in_the_file` | — | Automatisé seulement |
| Changer le retour pendant un cycle | `changing_only_return_does_not_shorten_natural_exit_grace`, `already_active_still_returns_to_explicit_frozen_plan` | — | Automatisé seulement |
| Plan cible ou retour manquant | `blocked_power_failure_only_retries_after_user_command`, `a_vanished_return_plan_is_not_called_the_missing_normal_plan` | — | Automatisé seulement |
| Lecture de processus indéterminée | `uncertainty_keeps_known_instance_then_blocks_at_ten_seconds`, `reused_pid_with_new_creation_does_not_keep_old_software_alive` | — | Automatisé seulement |
| Journal non inscriptible | `prepare_failure_forbids_power_write_and_periodic_retries` | — | Automatisé seulement |
| Coupure à chaque phase du journal | `recovery_matrix_respects_every_record_phase`, `failed_confirmation_keeps_prepared_record_for_restart`, `finalization_failure_recovers_without_repeating_acquisition` | `crash-restart-recovery` : arrêt forcé de l'application pendant un cycle, retour au plan d'avant le cycle au relancement | Vérifié pour la coupure après confirmation |
| Journal corrompu ou version future | `future_and_invalid_files_are_preserved`, `interrupted_temporary_file_preserves_last_valid_configuration` | — | Automatisé seulement |
| Batterie non autorisée | `battery_wait_says_no_plan_will_change` (présentation) | **Non exécutable** : la machine de référence est un PC fixe sans batterie | Ouvert |
| Veille et reprise | `suspend_keeps_cycle_and_resume_observes_current_processes`, `resume_broadcast_requests_a_fresh_snapshot` | **Non exécuté** : mettre le PC en veille interrompt la session d'essai sans personne pour le réveiller | Ouvert |
| Changement de session | `global_probe_refuses_a_second_controller_and_recovers_after_release`, `screen_lock_does_not_mark_the_session_inactive`, `a_lock_held_or_denied_reads_as_another_session_in_control` | **Non exécuté** : exige un second compte standard ouvert en même temps | Ouvert |
| Quitter avec retour refusé | `quit_after_failed_initial_write_leaves_the_original_plan`, `a_failed_return_while_quitting_keeps_the_active_plan_in_view` | — | Automatisé seulement |

Les journaux et mesures des essais réels sont dans
[behaviors/20261031-162909-fa4c40fa/](behaviors/20261031-162909-fa4c40fa/).

T027 reste ouverte. Batterie, veille et changement de session demandent un ordinateur portable
ou une personne présente devant le PC. Les lignes « Automatisé seulement » passent par un
adaptateur simulé, comme le guide l'admet pour les erreurs difficiles à reproduire.
