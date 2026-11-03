# Maquette Chron'O en Rust + Slint — consignes pour Claude Code

## Ce que c'est
Maquette d'interface de Chron'O (logiciel de GEC SportIdent), dépôt public `Chron-O/maquette` (GPLv3).
Elle remplace le prototype HTML (gelé dans `html/`). **Aucune fausse donnée** : l'app est vide tant que le générateur de test n'est pas lancé.
Le code pourra servir de point de départ à l'app (à confirmer avec Severin) : il doit donc rester propre.

## Règles strictes
- Travail sur une **branche**, puis pull request. **Rien n'est poussé sur GitHub sans l'accord explicite de Robin.**
- Ne jamais modifier `../chrono` ni `../essais-slint` (lecture seule).
- Aucune copie de code depuis chrono-swift, pyMeOS (AGPL) ou MeOS.
- Logo officiel uniquement : celui de `docs/identite/` de chrono-swift, texte en tracés Instrument Sans Bold (`ui/logo/`). Ne pas en dessiner d'autre.
- Robin découvre Rust : expliquer en français simple et court. **Avant d'écrire du code, reformuler et attendre l'accord.**
- Commits, code (noms compris) et README en anglais ; textes affichés à l'écran et échanges avec Robin en français.

## Architecture
- Un projet Cargo, un seul binaire `maquette`. Version web (WebAssembly) publiée sur GitHub Pages à chaque fusion sur `main`.
- **Rust donne les données, Slint affiche.** Aucune règle de course dans les `.slint` : ils affichent ce qu'on leur donne et renvoient les clics.
- Données de test : seulement via le générateur de `src/test_data.rs`, lancé depuis le panneau Démo (noms, clubs, puces inventés). Il remplace les appareils et événements réels pour tester l'interface. À l'arrêt, tout s'efface : si rien ne tourne, rien ne s'affiche.
- Point de départ : `ui/theme.slint`, `ui/components/`, `ui/icons/`, `ui/fonts/` repris de `../essais-slint` (déjà validés, puis renommés en anglais), et les leçons de `../essais-slint/LECONS-SLINT.md`.

## Navigation (cadrage validé)
- Comme le prototype : l'**Accueil** est une page qui défile, **sans onglets**, avec deux sections : Événements et Le club.
- Depuis Le club : Puces de prêt (stock de prêt), Boîtiers (inventaire du club), Coureurs et clubs (base fédérale). Retour par « ‹ Accueil » (« ‹ » seul en largeur étroite).
- Réglages : emplacement à décider, le prototype n'en a pas.
- Un événement : 6 onglets **toujours présents**, dans l'ordre : Circuits · Participants · Départs · Boîtiers · Direct · Résultats.
- Deux niveaux de navigation, pas plus. Le détail s'ouvre à côté (large) ou par-dessus (étroit, moyen).

## Disposition adaptative
La **largeur** décide de la disposition :
- **Étroite** (< 600 px, téléphone) : ligne pavé › nom › temps ; détail par-dessus ; onglets en icônes, mot sur l'actif seulement.
- **Moyenne** (600–1000 px, iPad portrait) : tableau allégé ; détail par-dessus ; onglets avec libellés.
- **Large** (> 1000 px, ordinateur, iPad paysage) : tableau complet ; détail à côté.

L'**appareil** décide des tailles, indépendamment : tactile (iOS, Android, web sur téléphone) 48 px, souris 36 px.
La largeur suit la taille réelle de la fenêtre, le clair/sombre suit le réglage du système.
Le panneau Démo permet seulement de forcer tactile/souris pour comparer, et de lancer le générateur.

## Placement (validé par Robin sur l'écran Circuits)
Une place par fonction, à tous les écrans, d'après `mise-en-page-universelle.md` et `departs-tirage-v2.md` § A de chrono-swift :
- **Ligne du haut** : réglages à gauche, alerte à droite, même hauteur, fixe. Le détail s'ouvre dans une bulle, par-dessus, sans rien décaler.
- **Colonne** (carte, fixe, s'il y a une navigation) : ses boutons à gauche, sa loupe à droite.
- **Au-dessus de la carte du contenu** : l'action d'ajout, seule.
- **En tête de la carte du contenu**, ce qui change ce qu'elle montre : affichage à gauche, recherche à droite, puis les filtres en petites pastilles. Une recherche est dans la zone où elle cherche.
- **Bande du bas**, collée à la fenêtre : chiffres à gauche, actions de la page à droite.
- Seul le contenu défile, et les titres d'un tableau restent en haut. Sur téléphone, c'est la page qui défile, jamais de côté ; la recherche est une loupe qui se déplie.
- Un aspect = un usage : ne pas donner le même dessin à un affichage et à un filtre.
- Avant de concevoir un écran : lire le prototype et les specs de chrono-swift (`docs/specs/`), en lecture seule.

## Design
- Système visuel : `references/charte.md` (copie de `../essais-slint/references/charte.md`). **Ne jamais la modifier** : signaler l'écart à Robin.
- Captures du prototype HTML : `references/captures/` — point de départ, pas un modèle à recopier.
- Règles absolues : pas de bordure sur un seul côté (sauf trait entre deux lignes) ; seulement les icônes de la charte ; aucun texte coupé ; clair et sombre ; jamais la couleur seule (icône + mot) ; aucune consigne inventée dans les textes ; toutes les valeurs viennent de `Theme`.
- Chaque information affichée doit servir : une information principale par ligne, une ou deux secondaires.

## À la fin de chaque écran
1. Captures automatiques dans `captures/` : **téléphone, iPad portrait, iPad paysage, ordinateur**, en clair et en sombre (8 images).
2. Mise à jour de `ECRANS.md` (statut) et des écarts avec la charte.
3. Dire à Robin en 3 lignes : comment lancer, quoi regarder, quelles décisions sont à prendre.
