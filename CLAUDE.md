# Maquette Chron'O en Rust + Slint — consignes pour Claude Code

## Ce que c'est
Maquette d'interface de Chron'O (logiciel de GEC SportIdent), dépôt public `Chron-O/maquette` (GPLv3).
Elle remplace le prototype HTML (gelé dans `html/`). Données **factices** uniquement.
Le code pourra servir de point de départ à l'app (à confirmer avec Severin) : il doit donc rester propre.

## Règles strictes
- Travail sur une **branche**, puis pull request. **Rien n'est poussé sur GitHub sans l'accord explicite de Robin.**
- Ne jamais modifier `../chrono` ni `../essais-slint` (lecture seule).
- Aucune copie de code depuis chrono-swift, pyMeOS (AGPL) ou MeOS.
- Pas de logo Chron'O tant que Robin ne l'a pas fourni.
- Robin découvre Rust : expliquer en français simple et court. **Avant d'écrire du code, reformuler et attendre l'accord.**
- Commits, code et README en anglais ; échanges avec Robin en français.

## Architecture
- Un projet Cargo, un seul binaire `maquette`. Version web (WebAssembly) publiée sur GitHub Pages à chaque fusion sur `main`.
- **Rust donne les données, Slint affiche.** Aucune règle de course dans les `.slint` : ils affichent ce qu'on leur donne et renvoient les clics.
- Données factices dans `src/donnees/` (noms, clubs, puces inventés).
- Point de départ : `ui/theme.slint`, `ui/composants/`, `ui/icones/`, `ui/polices/` repris de `../essais-slint` (déjà validés), et les leçons de `../essais-slint/LECONS-SLINT.md`.

## Navigation (cadrage validé)
- Niveau logiciel : Accueil · Événements · Boîtiers (inventaire du club) · Puces (stock de prêt) · Base coureurs (fédérale) · Réglages.
- Un événement : 6 onglets **toujours présents**, dans l'ordre : Circuits · Participants · Départs · Boîtiers · Direct · Résultats.
- Deux niveaux de navigation, pas plus. Le détail s'ouvre à côté (large) ou par-dessus (étroit, moyen).

## Disposition adaptative
La **largeur** décide de la disposition :
- **Étroite** (< 600 px, téléphone) : ligne pavé › nom › temps ; détail par-dessus ; onglets en icônes, mot sur l'actif seulement.
- **Moyenne** (600–1000 px, iPad portrait) : tableau allégé ; détail par-dessus ; onglets avec libellés.
- **Large** (> 1000 px, ordinateur, iPad paysage) : tableau complet ; détail à côté.

L'**appareil** décide des tailles, indépendamment : tactile (iOS, Android, web sur téléphone) 48 px, souris 36 px.
La largeur suit la taille réelle de la fenêtre, le clair/sombre suit le réglage du système.
Un réglage de démo permet seulement de forcer tactile/souris pour comparer.

## Design
- Système visuel : `references/charte.md` (copie de `../essais-slint/references/charte.md`). **Ne jamais la modifier** : signaler l'écart à Robin.
- Captures du prototype HTML : `references/captures/` — point de départ, pas un modèle à recopier.
- Règles absolues : pas de bordure sur un seul côté (sauf trait entre deux lignes) ; seulement les icônes de la charte ; aucun texte coupé ; clair et sombre ; jamais la couleur seule (icône + mot) ; aucune consigne inventée dans les textes ; toutes les valeurs viennent de `Theme`.
- Chaque information affichée doit servir : une information principale par ligne, une ou deux secondaires.

## À la fin de chaque écran
1. Captures automatiques dans `captures/` : **téléphone, iPad portrait, iPad paysage, ordinateur**, en clair et en sombre (8 images).
2. Mise à jour de `ECRANS.md` (statut) et des écarts avec la charte.
3. Dire à Robin en 3 lignes : comment lancer, quoi regarder, quelles décisions sont à prendre.
