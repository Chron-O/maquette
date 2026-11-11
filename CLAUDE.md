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
- Paramètres (brief zones-onglets, 10/11/2026) : une languette « ⚙ Paramètres : résumé » centrée, accrochée sous la bulle des onglets (sous l'en-tête sur les pages du club), qui se déplie par-dessus le contenu. Sans paramètres : pas de languette ; dans un événement, le contenu commence à la même hauteur ; sur une page du club (Coureurs et clubs), il monte sous l'en-tête. Les réglages d'un circuit ou d'un point de départ restent dans leur carte.
- Un événement : 5 onglets **toujours présents**, dans l'ordre : Circuits · Participants · Départs · Boîtiers · Résultats, dans une bulle blanche à cheval sur l'en-tête. Le **Direct** est un volet qui monte d'en bas (sa barre reste visible) ; son bouton est au bout de la bulle (Robin, 5/11 et 10/11).
- Deux niveaux de navigation, pas plus. Le détail (fiche) glisse depuis la droite par-dessus le contenu, à toutes les largeurs, sans voile ; sur téléphone il monte du bas, avec un voile.

## Disposition adaptative
La **largeur** décide de la disposition :
- **Étroite** (< 600 px, téléphone) : ligne pavé › nom › temps ; fiche qui monte du bas ; onglets en icônes, mot sur l'actif seulement. La page défile sous l'en-tête ; dès qu'elle a défilé, l'en-tête se tasse (nom plus petit, indicateurs en 3 points, languette rangée sous la bulle) ; tout revient en haut de la page (Robin, 11/11).
- **Moyenne** (600–1000 px, iPad portrait) : tableau allégé ; fiche par-dessus, à droite ; onglets avec libellés ; indicateurs à droite du nom de l'événement.
- **Large** (> 1000 px, ordinateur, iPad paysage) : tableau complet ; fiche par-dessus, à droite ; onglets avec icône et mot ; nom et indicateurs dans la bande verte, au-dessus de la bulle (Robin, 11/11).

L'**appareil** décide des tailles, indépendamment : tactile (iOS, Android, web sur téléphone) 48 px, souris 36 px.
La largeur suit la taille réelle de la fenêtre, le clair/sombre suit le réglage du système.
Indicateurs de l'en-tête : icône, point et nombre (Robin, 11/11).
Le panneau Démo permet seulement de forcer tactile/souris pour comparer, et de lancer le générateur.

## Placement (brief zones-onglets, 10/11/2026 ; remplace la version validée sur Circuits)
Une place par fonction, à tous les écrans :
- **Languette Paramètres** sous la bulle des onglets : le résumé des paramètres de la page ; elle se déplie par-dessus, rien ne se décale.
- **Volet gauche** flottant (marges, rayon 16, ombre légère ; contour fin en sombre) : la colonne de l'onglet, avec ses petits titres gris.
- **En tête de la carte du contenu** : à gauche l'action d'ajout en orange, puis les segments d'affichage ; à droite, dans cet ordre, l'alerte, le bouton Filtres (entonnoir), la recherche. Le « Voir » de l'alerte applique son filtre.
- **Filtres** : tous dans le bouton Filtres et sa bulle (un groupe par critère) ; les filtres actifs dans le bouton, chacun avec sa croix. Les affichages (Tableau / Lignes / Carte, Liste / Frise, Contrôle / Ramassage, « Mettre en avant ») restent des segments.
- **Bande du bas**, collée à la fenêtre : chiffres à gauche, actions de la page à droite.
- Un seul bouton orange par écran, hors actions de ligne : les boutons importants et souvent utilisés (Robin, 11/11).
- Seul le contenu défile, et les titres d'un tableau restent en haut. Sur téléphone, c'est la page qui défile, jamais de côté ; la recherche est une loupe qui se déplie.
- Un aspect = un usage : ne pas donner le même dessin à un affichage et à un filtre.
- Rien ne change de taille tout seul (lecture, alerte, appareil) : place réservée.
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
