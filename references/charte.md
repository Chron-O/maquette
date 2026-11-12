# Charte visuelle Chron'O (extrait pour les essais Slint)

Résumé des décisions de design (projet « Chrono », fichiers interface-*.md). Ne pas réinterpréter.

## Couleurs de base
| Rôle | Clair | Sombre |
|---|---|---|
| Marque (en-tête, onglet actif en clair) | vert forêt `#163628` | en-tête **neutre**, même gris-vert que les cartes (`#17241C`) : pas de vert forêt en sombre |
| Fond de page | `#F5F8F4` | `#0F1A14` |
| Carte | blanc `#FFFFFF`, ombre très légère tout autour | `#17241C` + bordure fine `#24352B` sur tout le contour |
| Surface interne (champs, boutons secondaires) | `#F0F5F1` | `#1F2F26` |
| Texte | `#163628` (ou quasi noir) | `#E8F0EA` |
| Texte secondaire | gris-vert | `#8FA398` |
| Onglet sélectionné | plein vert forêt, texte blanc | plein `#E8F0EA`, texte sombre |
| **Bouton principal** (une seule fois par écran, hors actions de ligne) | orange `#D9650C`, texte blanc, 18 gras | identique |
| Accent « vivant » (lecteur prêt, point du Direct) | menthe `#3DDC97` | menthe `#3DDC97` |
| Postes / circuits CO | violine `#B5179E` | violine sur bande claire |

## Familles d'états (la couleur dit la famille, l'icône le statut)
Toujours **icône + mot**. Toutes au même niveau d'intensité.
Icônes des statuts (décidé le 03/11) : OK `check` · PM `x` · Inconnue `circle-help` · Abandon `log-out` · En course : point plein, sans icône.

| Famille | Badge clair texte / fond | Pavé clair texte / fond | Sombre (texte, sur surface neutre) | Exemples coureurs |
|---|---|---|---|---|
| Valide | `#17663A` / `#DDF3E5` | `#0E4F2A` / `#A8E5BF` | `#7EE2A0` | OK |
| En cours (animé) | `#0B6B5A` / `#D3F5EC` | `#074B3F` / `#A7EBDA` | `#5FE0C8` | En course |
| À vérifier | `#6B5200` / `#FFF1B8` | `#4D3B00` / `#FFE07A` | `#FFE066` | Inconnue (matériel : pile faible) |
| Problème | `#D92D20` / `#FDE4E1` | `#A3170D` / `#FFCBC4` | `#FF6B5E` | PM, Disqualifié |
| Inactif | `#4F5A63` / `#ECEFF1` | `#39434B` / `#D5DBDF` | `#B4BEC6` | Pas encore parti, Abandon, Absent |

En sombre : **surfaces toutes neutres**, la couleur seulement sur texte/icône du statut, contour d'action, bouton orange, point menthe.

## Typographie
- **IBM Plex Sans** (licence OFL, à embarquer). IBM Plex Mono pour données brutes si besoin.
- Échelle : 48 temps géant · 22 titre d'écran · 17 titre de bloc · 16 texte courant · 14 libellé · 13 aide (minimum).
- Graisses : 400 lecture · 500 libellés/onglets · 600 titres de bloc, temps en liste · 700 titres d'écran, temps géant, bouton principal.
- Chiffres à largeur fixe pour tous les temps.

## Tailles et formes
- Zones à toucher : 48 px téléphone · 56 px actions de terrain · 36 px ordinateur.
- Style « vif » : plat, aucune ombre sur les boutons, retour au toucher par changement de teinte.
- Boutons et onglets en **pilule** ; cartes rayon 16 ; champs rayon 12, 36 px de haut sur ordinateur, 48 px sur téléphone ; bouton secondaire blanc à contour fin, rayon 10.
- Bouton principal (décidé le 03/11) : 36 px de haut sur ordinateur, 48 px sur téléphone, texte 18 gras.
- Action dans une ligne, ex. « Attribuer » (remplacé le 12/11/2026) : **bouton de ligne unique**, pilule à contour, orange pour une chose à faire (Attribuer, Rendre, Réserver), vert foncé pour aller voir ailleurs (Voir ses participants ›), jamais plein ; texte 16 gras ; dessin 32 px sur ordinateur, 40 px sur téléphone ; zone à toucher inchangée (36 et 48 px).
- Bouton discret (décidé le 03/11) : texte souligné, sans fond ni contour.
- **Interdit : bordure ou liseré sur un seul côté.** Exception : le trait fin qui sépare deux lignes d'une liste ou d'un tableau (décidé le 03/11).
- Champ : fond `#F0F5F1` sans bordure au repos ; en saisie fond blanc + contour complet vert forêt 2 px ; en erreur contour rouge + message dessous.
- Champ en sombre (décidé le 03/11) : repos fond `#1F2F26` sans bordure ; en saisie fond `#17241C` + contour complet menthe `#3DDC97` 2 px, texte `#E8F0EA` ; en erreur même fond + contour `#FF6B5E` + message dessous.

## Listes : deux formes (décidé le 03/11)
**Ordinateur : tableau à colonnes** (voir `08-direct.png`) : État (pastille) · Nom · Catégorie · Temps · Place · Lu à · Action.

**Téléphone : ligne de liste.** Gauche › centre › droite = **statut › qui et où › détails**.
- Marges (décidé le 03/11) : fond teinté et traits entre les lignes vont jusqu'au bord de la carte ; le contenu des lignes garde la même marge que le titre de la carte, à gauche et à droite (24 px sur ordinateur, 16 px sur téléphone).
- Pavé de statut de largeur fixe, mot complet (OK, PM, En course, Abandon, Inconnue).
- Centre : nom (17, 600) + circuit (petite pastille carrée de couleur + nom).
- Droite : temps (22, 700) + une précision (place ; « manque le 38 » en rouge).
- Temps d'un PM, dans les deux formes (décidé le 03/11) : grisé (texte secondaire), moins gras, barré d'un trait : il ne compte pas au classement. Écart assumé avec `08-direct.png`.

## Animations
- « En cours » : point qui respire. Lecture / synchronisation : icône qui tourne.
- Coupées si le système demande moins de mouvements.

## Icônes
- **Lucide** (ISC) : grille 24, trait 2 px, bouts arrondis, contour seul. Utilisées : `users` `radio` `trophy` `check` `x` `footprints` `log-out` `circle-help` (fichier `circle-question-mark` depuis Lucide 1.67) `settings` `plus` `triangle-alert` `pin` `pin-off` `monitor-smartphone` `tv` `app-window` `laptop` `smartphone` `qr-code` `funnel` (bouton Filtres, validé le 10/11/2026) `chevron-down` (languette Paramètres, validé le 10/11/2026) `lock` (heure de départ verrouillée, validé le 12/11/2026) `search` (loupe de la recherche sur téléphone, validé le 12/11/2026).
- **Icônes CO** (même style, viewBox 0 0 24 24, stroke 2, round) :
  - Départ : `M12 4 21 19H3Z`
  - Poste : cercle (12,12) r7
  - Arrivée : cercles (12,12) r8 et r4.5
  - Circuit : `M3 6.5 9 4l6 2.5L21 4v13.5L15 20l-6-2.5L3 20Z` + `M9 4v13.5` + `M15 6.5V20`
  - Balise : rect (4,2) 16×14 r1.5 + `M4 2 20 16` + `M12 16v6`
  - Puce : rect (4,9) 11×6 r3 + `M15 12h3` + `M19.5 8.5a5 5 0 0 1 0 7` + `M17.5 10a2.4 2.4 0 0 1 0 4`
  - Boîtier : rect (7,2.5) 10×19 r3 + cercle (12,8) r2.5 + voyant (12,15.5)
