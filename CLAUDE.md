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
- Paramètres (brief zones-onglets, 10/11/2026) : une languette « ⚙ Paramètres : résumé » centrée, accrochée sous la bulle des onglets (sous l'en-tête sur les pages du club), qui se déplie par-dessus le contenu. Sans paramètres : pas de languette ; dans un événement, le contenu commence à la même hauteur ; sur une page du club (Coureurs et clubs), il monte sous l'en-tête. Les réglages d'un circuit ou d'un point de départ restent avec lui : sa bulle de titre et, glissée dessous, une languette « ⚙ Réglages : résumé », au même dessin et au même dépliage que les Paramètres, posées sur le fond de la page au-dessus de sa carte (Robin, 11/11).
- Un événement : 5 onglets **toujours présents**, dans l'ordre : Circuits · Participants · Départs · Boîtiers · Résultats, dans une bulle blanche à cheval sur l'en-tête. Le **Direct** est un volet qui monte d'en bas (sa barre reste visible) ; son bouton est au bout de la bulle (Robin, 5/11 et 10/11).
- Deux niveaux de navigation, pas plus. Le détail (fiche) glisse depuis la droite par-dessus le contenu, à toutes les largeurs, sans voile ; sur téléphone il monte du bas, avec un voile.

## Disposition adaptative
La **largeur** décide de la disposition :
- **Étroite** (< 600 px, téléphone) : ligne pavé › nom › temps ; fiche qui monte du bas ; onglets en icônes, mot sur l'actif seulement. La page défile sous l'en-tête ; dès qu'elle a défilé, l'en-tête se tasse (nom plus petit, indicateurs en 3 points, languette rangée sous la bulle) ; tout revient en haut de la page (Robin, 11/11).
- **Moyenne** (600–1000 px, iPad portrait) : tableau allégé ; fiche par-dessus, à droite ; onglets avec libellés ; indicateurs à droite du nom de l'événement s'il tient sur une ligne avec eux, sinon dessous, à droite (de même sur téléphone).
- **Large** (> 1000 px, ordinateur, iPad paysage) : tableau complet ; fiche par-dessus, à droite ; onglets avec icône et mot ; nom et indicateurs dans la bande verte, au-dessus de la bulle (Robin, 11/11).

L'**appareil** décide des tailles, indépendamment : tactile (iOS, Android, web sur téléphone) 48 px, souris 36 px.
La largeur suit la taille réelle de la fenêtre, le clair/sombre suit le réglage du système.
Indicateurs de l'en-tête : icône, point et nombre (Robin, 11/11).
Le panneau Démo permet seulement de forcer tactile/souris pour comparer, et de lancer le générateur.

## Placement (brief Cowork volet / tête de carte, Robin 11/11/2026 ; remplace la version du 10/11)
Une place par fonction, à tous les écrans (inspirée de `mise-en-page-universelle.md` de chrono-swift, mais c'est elle qui fait foi) :

| Zone | Rôle | Contenu |
|---|---|---|
| **Volet gauche** | Ce qu'on regarde : une seule liste, sans titre en tête | Des sous-onglets (des tâches : Contrôle / Ramassage, Inventaire), ou ce qu'on a créé pour la course (circuits, groupes) |
| **Sous-zones du volet** (petit titre gris), dessous | D'autres façons de choisir ce que montre la carte | Ex. « Puces prêtées · 4 » montre les 4 participants concernés. Jamais d'information seule, jamais d'action de la page |
| **Tuile du lecteur** | À part, hors du volet | En bas à gauche, sous le volet, comme les lecteurs du Direct. Le volet commence partout à la même hauteur. Téléphone : la tuile reste au-dessus de la liste |
| **Tête de carte, à gauche** | Ce qu'on ajoute | Ajout en orange, Importer à côté |
| **Tête de carte, au centre** | Comment on regarde | Les affichages en segments (Liste / Frise, Liste / Métro / Géographique…), toujours centrés dans la carte |
| **Tête de carte, à droite** | Ce qu'on réduit | Alerte, Filtres, recherche, dans cet ordre |
| **Bande du bas** | Les sorties | Chiffres à gauche ; Imprimer, Exporter, Publier à droite |

- Une place vide reste vide : rien d'autre n'y glisse. Largeur insuffisante (téléphone, iPad portrait) : les affichages passent sur une 2ᵉ ligne, centrée.
- Un bouton d'ajout se met en tête de la liste qu'il remplit : « + Groupe » dans le volet de Départs, « Nouveau circuit » dans celui de Circuits, « + Poste » en tête de carte.
- Un choix n'a qu'une place : ce qui est dans le volet sort de l'entonnoir, et inversement. Le volet choisit l'ensemble, les Filtres le réduisent.
- Même construction partout : la tête de carte est une bande à part ; chaque objet (un circuit, un bloc de départs) est une carte avec sa bulle de titre (plus large que sa languette) et sa languette Réglages, posées sur le fond de la page ; titre, languette et titres de colonnes restent en haut pendant que ses lignes défilent.
- **Languette Paramètres** sous la bulle des onglets : le résumé des paramètres de la page ; elle se déplie par-dessus, rien ne se décale. Ouverte, un clic ailleurs la referme sans rien sélectionner.
- **Lecteur** au format moyen partout : nom et n° sur une ligne, l'état et l'épingle dessous, puis une zone fixe de 2 lignes (consigne, ou numéro lu et verdict, ou boutons d'une question ; alors le numéro remonte à la place de l'état). Écrans de boîtiers : un liseré orange épais ; l'état (lecture, inconnu, débranché) garde sa couleur.
- **Filtres** : tous dans le bouton Filtres et sa bulle (un groupe par critère) ; les filtres actifs dans le bouton, chacun avec sa croix. Le « Voir » de l'alerte applique son filtre.
- Un seul bouton orange par écran, hors actions de ligne.
- Seul le contenu défile. Sur téléphone, c'est la page qui défile, jamais de côté, sauf les Colonnes de Classements (seule exception, ordinateur et téléphone) ; la recherche est une loupe qui se déplie.
- Un aspect = un usage : ne pas donner le même dessin à un affichage et à un filtre.
- Rien ne change de taille tout seul (lecture, alerte, appareil) : place réservée.
- Ce qui s'ouvre ou se ferme est animé, jamais d'un coup : `Unfold` pour une ligne qui se déplie, fondu et glissé court pour une bulle (`Courses.close-bubble()` pour la fermer), `Theme.*-duration` (0 si mouvements réduits).
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
