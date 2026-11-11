# Écrans de la maquette

Un écran à la fois, dans cet ordre. On ne passe au suivant qu'après validation de Robin.
Chaque écran : 3 dispositions (étroite, moyenne, large), tactile et souris, clair et sombre.

| # | Écran | Statut |
|---|---|---|
| 0 | Mise en place (dépôt, projet, publication web) | validé par Robin ; publication web à tester à la première fusion sur `main` |
| 1 | Accueil | validé par Robin |
| 2 | Course › Circuits | à revoir par Robin (zones et onglets, 10/11) ; avant : accepté par Robin sans relecture détaillée ; Mixte à maquetter, modification des circuits à faire plus tard |
| 3 | Course › Participants | à revoir par Robin (zones et onglets, 10/11) ; avant : accepté par Robin sans relecture détaillée ; nouveau participant et attribution de puce à faire plus tard |
| 4 | Course › Départs | à revoir par Robin (zones et onglets, 10/11) ; avant : accepté par Robin sans relecture détaillée ; réglage des groupes, vrai tirage et modifications à faire plus tard |
| 5 | Course › Boîtiers | à revoir par Robin (zones et onglets, 10/11) ; avant : accepté par Robin sans relecture détaillée ; Programmer, Choisir un boîtier, Corriger les temps et boîtiers spéciaux à faire plus tard |
| 6 | Course › Direct | à revoir par Robin (zones et onglets, 10/11) ; avant : accepté par Robin sans relecture détaillée ; Arrivée manuelle, ticket et modification du participant à faire plus tard |
| 7 | Course › Résultats | à revoir par Robin (zones et onglets, 10/11) ; avant : accepté par Robin sans relecture détaillée ; sous-onglets, égalités, Temps maximal, Imprimer et Exporter à faire plus tard |
| 8 | Puces (stock de prêt) | à revoir par Robin (zones et onglets, 10/11) ; avant : accepté par Robin sans relecture détaillée ; Inventaire, Prêter, Ajouter, parcs et sélection multiple à faire plus tard |
| 9 | Boîtiers (inventaire du club) | à revoir par Robin (zones et onglets, 10/11) ; avant : accepté par Robin sans relecture détaillée ; Inventaire, Ajouter, mémoire, parcs et sélection multiple à faire plus tard |
| 10 | Coureurs et clubs (base fédérale) | à revoir par Robin (zones et onglets, 10/11) ; avant : accepté par Robin sans relecture détaillée ; import FFCO, contrôle des doublons, fusion et modification à faire plus tard |


## Zones et onglets (brief du 10/11/2026, `../consignes-maquette/zones-onglets/`)
Appliqué aux écrans 2 à 10 : bulle d'onglets à cheval sur l'en-tête, languette « Paramètres : … », volet gauche flottant, fiche qui glisse depuis la droite (du bas sur téléphone), bouton Filtres avec sa bulle, action d'ajout orange à gauche dans la carte, alerte · Filtres · recherche à droite.

### Choix et écarts à relire
- **Direct** : gardé en volet (décision de Robin du 10/11), donc 5 onglets dans la bulle et le bouton Direct à son bout ; pas de languette.
- **Languette sur téléphone** : « Paramètres » seul, le résumé se lit en l'ouvrant (Robin, 10/11) ; flèche `chevron-down` ajoutée à la charte avec `funnel`.
- **Filtres actifs sur téléphone** : l'entonnoir et leur nombre, sans les étiquettes, qui ne tiendraient pas.
- **Groupes de filtres ajoutés** (validés par Robin le 11/11) : Participants « À vérifier » (Doublons possibles, Catégorie inconnue) et l'état « PM », pour que chaque « Voir » de l'alerte ait son filtre ; Direct « Lectures : À traiter » ; Coureurs, un seul groupe « Coureurs » (licence et pratique mêlées, un seul choix).
- **Bulles des filtres et de l'alerte** : collées à leur bouton, comme les autres bulles (variante B). Ramenées à 340 px de large, comme les captures du brief ; chaque pastille de filtre a la largeur de son mot et passe à la ligne s'il le faut (Robin, 11/11 : « beaucoup trop grandes »). Sur téléphone, la bulle des filtres défile si elle touche la bande du bas.
- **Bulle d'onglets** : affinée (Robin, 11/11) : 4 px de blanc autour des onglets, ombre plus douce ; la zone de toucher garde 48 px.
- **Circuits sur téléphone** : Tableau / Lignes passent sous l'action, une seule ligne débordait de l'écran.
- **En-tête tassé sur téléphone** (Robin, 11/11) : la page défile sous l'en-tête et la bulle ; une fois qu'elle a défilé, le nom passe en plus petit, les indicateurs deviennent 3 points et la languette se range sous la bulle ; tout revient en haut de la page seulement. Écart avec la charte : les 3 points sont de la couleur seule (choix de Robin) ; leur état reste lu par les lecteurs d'écran. Capture : `03-participants-phone-scrolled`.
- **En-tête sur ordinateur** : nom et indicateurs dans la bande verte, à 8 px au moins de la bulle, avec moins de vide en haut (Robin, 11/11).
- **En-tête sur iPad portrait** : les 3 indicateurs l'un sous l'autre, à droite du nom, pour garder une seule rangée (Robin, 11/11).
- **Boîtiers** : « Boîtier spécial » en orange (Robin, 11/11 : l'orange va aux boutons importants et souvent utilisés). Contrôle / Ramassage reste un affichage, dans le volet.
- **Circuits** : pas de filtre ; « Mettre en avant » reste un affichage. Les réglages d'un circuit restent dans sa carte.
- **Résultats** : le choix du classement reste dans le volet ; seules les catégories d'un classement par circuit passent dans les Filtres.
- **Puces et Boîtiers du club** : le choix du parc reste un affichage (volet, ou pastilles sur téléphone) ; la languette donne la description du parc.
- **Coureurs et clubs** : pas de paramètres, donc pas de languette ; le contenu monte juste sous l'en-tête, la page étant indépendante (Robin, 11/11) ; « Ajouter un coureur » passe en orange.
- **Fiche** : ses boutons orange passent en blanc (« Garder le premier résultat ») ; les actions de ligne restent orange.
- **Non testé** : Échap au clavier, iPad paysage, la fiche ouverte sur toutes les pages du club.

## 0 — Mise en place
- Ranger le prototype HTML dans `html/` (il reste en ligne à `…/maquette/html/`) ; mettre à jour le README.
- Projet Cargo à la racine ; reprendre thème, composants, icônes, polices des essais.
- Une coquille vide qui navigue, comme le prototype : l'Accueil (sections Événements et Le club, sans onglets), les pages du club et les 6 onglets d'un événement, chaque page avec son titre seulement.
- Les 3 dispositions (selon la largeur de la fenêtre) et le réglage de démo (tactile/souris seulement ; le clair/sombre suit le système).
- Action GitHub qui construit la version web et la publie sur GitHub Pages.
- Captures automatiques (8 par écran).
- **Réussi si** : la coquille tourne sur Mac, sur iPhone et dans le navigateur ; la navigation est identique partout ; aucune valeur de design en dur.

## 1 — Accueil
- En-tête : logo officiel, sans nom de club ; les 3 indicateurs (version courte sous 1000 px). Pour l'instant sur l'Accueil seulement.
- Sans générateur : app vide. Pas de grande carte ni de Direct, « Aucun événement », club à 0, indicateurs « aucun ».
- Générateur (panneau Démo, Données de test : Marche) : événements, chiffres du club, lecteurs, et une lecture toutes les 3 s dans le Direct. Arrêt : tout s'efface.
- Captures : `01-home-*` (vide) et `01-home-test-data-*` (tirage fixe).

### Écarts et points ouverts de l'écran 1
- **Logo** : `ui/logo/logo.svg` reprend le dessin de chrono-swift, avec le texte en tracés depuis Instrument Sans Bold (décision de Robin) : aucune police à installer. Les PNG de `docs/identite` de chrono-swift ont été rendus avec une police de secours et n'ont pas changé : les lettres diffèrent un peu.
- **Bouton orange** : un seul (« Ouvrir l'événement »), comme la charte. Le prototype en a deux.
- **Pastilles** : icône ajoutée (`check`, `triangle-alert`), absente du prototype. Les grises restent sans icône : aucune ne convient dans la charte.
- **Tailles hors échelle de la charte** : 34 (nom d'événement, chiffres du club), 28 (compteurs), 24 (date), reprises de `01-accueil.png`.
- **Rayon** : 16 comme la charte, sur la carte blanche et le Direct (20 dans le prototype).
- **Carte en sombre** : gardée claire, comme une carte papier. Ses couleurs sont dans `ui/map/hero-map.svg`, c'est une illustration.
- **Disposition moyenne** : absente du prototype, proposée ici. Carte blanche et Direct côte à côte dès 760 px.
- **Téléphone** : Direct sous la carte blanche, sans le chevauchement du prototype.
- **« 2 à traiter ›** ouvre l'onglet Direct ; le prototype ouvre un tiroir, qui n'existe pas ici (voir écran 6).
- **Sans effet pour l'instant** : Rejoindre, Nouvel événement, Voir tous, indicateurs.

## 2 — Circuits
- « Tous les postes » : liste des circuits, Tableau / Lignes / Carte, tri, recherche, « Mettre en avant », fiche du poste.
- Tout est en lecture seule pour l'instant : Importer, Nouveau, Ajouter, Neutraliser, Boîtier double, Imprimer et Exporter sont visibles, sans effet.
- Vue d'un circuit, d'après `editeur-parcours.md` de chrono-swift : Liste, Métro, Géographique, réserve, légende, « Montrer les autres circuits », fiche du poste. Deux zones de réglages : « Tous les circuits » en haut, le circuit en tête de sa carte.
- Formats du générateur : le dernier circuit de la liste est au score, celui d'avant en ordre libre, les autres en ordre imposé.
- Captures : `02-courses-*` (tirage fixe, en Tableau), `02-course-*` (Circuit A, en Métro, ordre imposé) et `02-course-score-*` (Circuit C, en Métro).

### Écarts et points ouverts de l'écran 2
- **Fiche sur téléphone** : par-dessus, comme `CLAUDE.md`. Le prototype l'ouvre dans la ligne.
- **Fiche en large** : absente tant qu'aucun poste n'est choisi. Le prototype affiche une phrase d'aide, retirée comme les autres consignes.
- **Icônes absentes de la charte, donc retirées** : importer, imprimer, exporter, et les petites flèches ▾.
- **Loupe, ajoutée à la demande de Robin** : absente de la charte, dessin repris du prototype, sans mot à côté. Dans la colonne des circuits à toutes les tailles et dans la carte du contenu sur téléphone ; elle déplie le champ. Sur iPad et ordinateur, la carte du contenu garde un champ de largeur limitée.
- **Alerte en pastille, idée de Robin** : ni le prototype ni les specs ne la décrivent. Même hauteur que les réglages ; son détail s'ouvre dans une bulle, une ligne par poste, chacune avec son « Le voir ».
- **Réglages en bulle** : comme le prototype. Sans effet pour l'instant, et « Tronçons non comptés » sans valeur, le générateur ne produit pas cette donnée.
- **Colonne des circuits** : 260 px (270 en tactile) au lieu des 250 du prototype, pour que ses boutons et la loupe tiennent.
- **Colonne de droite** : les specs du 16 octobre la retirent ; Robin garde la fiche du poste à droite en large, par-dessus sinon. Elle reste plus légère que l'ancienne colonne.
- **Lignes** : les postes passent à la ligne au lieu de défiler de côté, à toutes les tailles.
- **Tri** : petit triangle dessiné, ▲ n'étant pas dans la police.
- **Poste dans aucun circuit** : contour plus clair au lieu de pointillés, que Slint ne dessine pas.
- **Fiche, avant et après** : Départ et Arrivée dessinés avec leurs symboles de la charte (`co-start`, `co-finish`).
- **« 1 tronçon non compté »** : pas dans le résumé, le générateur ne produit pas cette donnée.

- **Vue d'un circuit, écarts avec la spec** : pas de poignée ni de corbeille (rien ne se modifie encore, et ces icônes ne sont pas dans la charte) ; pas de « + » entre les postes ; pas de phrase « Cliquez un poste… » ; pas de fiche vide en pointillés. Bouton « + Poste » au-dessus de la carte, repris du prototype. Sur iPad en portrait, le résumé de « Tous les circuits » reste dans la bulle.
- **Serpentin du Métro** : 5 stations par rangée comme la spec, moins si la largeur manque, pour ne jamais défiler de côté.
- **Bleu des postes obligatoires** : absent de la charte, valeur de la spec (`#1F5FBF`), identique en clair et en sombre car les cercles restent sur fond clair.
- **Ordre libre et Score, ajouts à la spec** : Départ et Arrivée au-dessus et en dessous de la zone en Métro, pour ne pas défiler de côté. Dans Tous les postes, cercle vide à la place du rang, et « libre » dans la fiche, sans les postes avant et après.
- **Résumé de « Tous les circuits »** : affiche le réglage général (« Ordre imposé »), même sur un circuit libre.
- **Postes présents deux fois, Mixte, tronçons non comptés** : pas encore faits ; le générateur ne les produit pas.

## 3 — Participants
- D'après `02-participants.png`, `participants-resultats.md` et `mise-en-page-universelle.md` de chrono-swift : encart de lecture et puces prêtées en colonne, tableau trié par les titres, filtres par circuit, recherche, fiche avec un tracé par passage. Lecture seule : Nouveau participant, Attribuer, Vérifier, Modifier, Arrivée manuelle, Historique, Importer, Imprimer et Exporter sont visibles, sans effet.
- Générateur : 30 à 60 vrais coureurs par événement. L'accueil et le Direct en dépendent : une lecture fait revenir un coureur en course.
- Captures : `03-participants-*` (tirage fixe).

### Écarts et points ouverts de l'écran 3
- **Fiche** : à droite en large, par-dessus sinon, comme la fiche du poste (décision de Robin). Le prototype l'ouvre sous la ligne, les specs dans une modale accrochée à la ligne. Affichée en lecture, sans champs modifiables, et sans « Enregistré à chaque changement ».
- **Importer** : dans la bande du bas, comme les specs ; le prototype le met près de la recherche.
- **Colonne** : « + Ajouter un panneau » retiré.
- **Bouton « … »** de ligne retiré, ses actions sont dans la fiche.
- **Tableau allégé selon la place** : sans Club ni Départ, et « Attribuer » au lieu de « Attribuer une puce », dès que la largeur manque, y compris sur iPad paysage avec la fiche ouverte.
- **Téléphone** : dernière colonne avec le temps si le coureur est revenu, sinon l'heure de départ ; bande raccourcie à « N participants · N sans puce ».
- **Icône de l'encart de lecture** : celle de l'Arrivée, la charte n'ayant pas d'icône pour la cible de la spec.
- **Bouton de ligne** : 16 gras comme la charte, plus gros que dans le prototype.
- **Pastilles par circuit** : pas de regroupement par couleur de la spec, le générateur nomme ses circuits A, B, C.
- **Non testé** : saisie dans la recherche, défilement du tableau au doigt.

## 4 — Départs
- D'après le prototype, `departs-tirage-v2.md`, `departs-horaires.md` et `lot4-departs.md` de chrono-swift : groupes en colonne, Liste par circuit et Frise, réglages en bulles, fiche du coureur.
- Fonctionnent, sur les données de test seulement : tirage en aperçu (Refaire, Abandonner, Appliquer), verrou, publication. Le reste est visible, sans effet.
- Captures : `04-starts-*` (tirage fixe, en Liste).

### Écarts et points ouverts de l'écran 4
- **Pas de sous-onglets** : décision de Robin, comme le prototype et les deux niveaux de `CLAUDE.md`. Les specs prévoient Groupes, Tirage, Frise, Liste de départ.
- **Tirage simplifié** : au hasard dans chaque circuit, sans toucher aux coureurs verrouillés ou partis. Clubs, groupes, demandes et « mise à jour minimale » de la spec ne sont pas appliqués.
- **Cadenas** : absent de la charte, ajouté à la demande de Robin, dessin Lucide.
- **Vacant** : contour clair au lieu de pointillés, que Slint ne dessine pas.
- **Bouton « ⋯ »** de ligne remplacé par la fiche.
- **« Diffuser »** retiré : les écrans publics se gèrent depuis l'indicateur de l'en-tête.
- **Phrases d'aide du prototype** retirées.
- **Téléphone** : Liste seule, sans Frise ni groupes.
- **Tableau allégé selon la place** : sans Club ni Puce. Colonne « Cat. » abrégée. « Nouvel horaire » remplace « Pas encore parti ».
- **Générateur** : les heures de départ n'influencent pas qui rentre ; un coureur parti tard peut être arrivé avant un parti tôt.
- **Pas faits** : réglage d'un groupe, premier poste commun, pauses, aperçu détaillé de la publication.
- **Non testé** : saisie dans la recherche, bouton de verrou, Refaire et Abandonner, tri, pastilles S1/S2, aperçu sur téléphone et iPad, cases déplacées et défilement de la Frise.

## 5 — Boîtiers
- D'après le prototype, `mise-en-page-universelle.md` et `charte-ui.md` de chrono-swift : encart de lecture, Contrôler et Ramasser en colonne, jauge, tableau par besoin (postes, départs, arrivées), filtre, recherche, fiche, alerte et réglages en bulles.
- Fonctionnent, sur les données de test seulement : lecture simulée (Démo › « Lire un boîtier », visible sur cet onglet), Oui en Contrôler, Ajouter à la course et Tout ajouter en Ramasser, Voir dans l'alerte. Le reste est visible, sans effet.
- Captures : `05-stations-*` (tirage fixe, en Contrôler).

### Écarts et points ouverts de l'écran 5
- **Carte de lecture** : elle décale le tableau quand elle apparaît, contraire à « rien ne saute » de la spec, comme l'aperçu du tirage de Départs.
- **« Voir » de l'alerte** : les trois filtrent sur tous les besoins à régler, pas sur leur seul type.
- **Boutons de ligne sans effet** : Programmer, Choisir un boîtier, Corriger les temps. Ils deviennent « Régler » dans le tableau allégé et sur téléphone.
- **En Ramasser** : les besoins encore « À sortir », « À programmer » ou « Manque » restent dans la liste, avec leur état. Filtre « Dehors » ajouté.
- **Jauge en Ramasser** : la barre compte les boîtiers rentrés, le gros chiffre ceux qui restent dehors, comme le prototype.
- **Téléphone** : n° de boîtier à côté du nom, état dessous, pour tenir à 320 px.
- **Numéro du lecteur de l'encart** : inventé par la maquette.
- **Référence** : les captures du prototype montrent l'ancien essai à sous-onglets ; le code du prototype fait foi ici.
- **Non testé** : boutons Oui, Ajouter à la course, Tout ajouter et Voir ; recherche, tri ; bulle des réglages ; fiche d'un besoin sans boîtier ; Ramasser sur téléphone et iPad. Web et iPhone compilent, sans avoir été ouverts.

## 6 — Direct
- D'après `08-direct.png`, le code du prototype, `live-fil.md` et `fiche-coureur-live.md` de chrono-swift : lecteurs et « La course en direct » en colonne, dernière lecture en grand, fil de toutes les lectures, filtres par circuit et par lecteur, recherche, fiche, alerte en bulle.
- Fonctionnent, sur les données de test seulement : lectures toutes les 3 s, Attribuer une puce inconnue, Garder le premier résultat d'une puce lue deux fois, listes des chiffres, filtres. Arrivée manuelle, Chercher un autre participant, C'est un autre coureur, Imprimer le ticket et Modifier le participant sont visibles, sans effet.
- Générateur : les coureurs rentrés sont dans le fil dès le départ, avec au moins un PM et deux lectures à régler ; d'autres arrivent ensuite de temps en temps.
- Captures : `06-live-*` (tirage fixe).

### Écarts et points ouverts de l'écran 6
- **Volet, pas onglet** (Robin, 5/11, confirmé le 10/11) : le Direct monte d'en bas, comme le prototype ; sa barre reste visible, son bouton est au bout de la bulle des onglets.
- **La course en direct** : vient de la spec, absente du prototype. « À régler » reste seulement dans l'alerte, pour ne pas le montrer deux fois.
- **« en course »** comme le prototype, au lieu de « en forêt » de la spec.
- **Noter l'heure, Arrivée sans puce, Mémoire des boîtiers** (`live-barre-actions.md`) : absents, comme le prototype. Pas de bouton orange hors des lignes.
- **Dernière lecture** : reste jusqu'à la suivante, comme la spec. Le prototype l'efface après 15 s.
- **Fiche** : glisse depuis la droite par-dessus le fil, à toutes les largeurs ; monte du bas sur téléphone (brief zones-onglets). La spec prévoit presque toute la fenêtre, avec Pourquoi ?, décisions d'arbitre et Passer, non faits.
- **Temps de passage** : le générateur ne garde pas les poinçons. Temps répartis régulièrement sur le temps total ; poste manquant d'un PM tiré d'après ce temps.
- **« réglée à 10:22 »** sans « par <nom> » : pas de personnes dans la maquette.
- **Attribuer et Régler** : les mots du prototype, au lieu de « Qui est-ce ?… » de la spec.
- **Filtre des lecteurs** : « 2 lecteurs » au lieu de « Tous », pour éviter deux « Tous » côte à côte.
- **Tableau** : pas de tri par les titres, le fil reste dans l'ordre des lectures. Bouton « ⋯ » de ligne remplacé par la fiche. Quand la place manque, catégorie sous le nom et place à côté du temps.
- **Téléphone** : tracé des postes seulement dans la fiche ; pas de bouton dans la grande carte, qu'on touche ; pastilles au lieu des tuiles de lecteur.
- **Bulle ouverte depuis l'accueil** : sa hauteur est une valeur fixe, l'alerte n'étant pas encore placée à ce moment.
- **Non testé** : recherche, filtre par lecteur, Garder le premier résultat, le réglage et la bulle sur téléphone et iPad, fiche d'un coureur sans lecture, fil en marche sur iPad et téléphone. Web et iPhone compilent, sans avoir été ouverts.

## 7 — Résultats
- D'après le prototype et ses captures : classements en colonne, classement choisi en tableau (place, nom, club, temps, écart, état), réglages en bulle, Imprimer et Exporter dans la bande du bas.
- Fonctionnent, sur les données de test seulement : choix du classement, Classer par catégorie ou par circuit, pastilles de catégorie en mode circuit, recherche, fiche partagée avec le Direct, classements recalculés à chaque lecture. Temps maximal, Imprimer et Exporter sont visibles, sans effet.
- Captures : `07-results-*` (tirage fixe). `06-live-*` refaites, les places ayant changé.

### Écarts et points ouverts de l'écran 7
- **Pas de sous-onglets** : Analyse et Remise des prix absents. Pas de vue Colonnes.
- **Places du Direct et de la fiche** : elles suivent maintenant le réglage « Classer », pour donner le même chiffre que le classement. Le Direct, déjà validé, montre donc la place par catégorie au réglage de départ.
- **Égalités** : deux temps identiques ont deux places différentes. La vraie règle reste à faire.
- **Classement par catégorie** : ne garde que les passages sur le premier circuit du coureur. Les autres passages apparaissent seulement par circuit.
- **Fiche** : refermée au changement d'onglet. Elle montre la dernière lecture du coureur : pour quelqu'un à deux passages, ce peut être l'autre passage que celui de la ligne, et les deux lignes se surlignent ensemble.
- **Recherche** : limitée au classement affiché.
- **Dernière lecture** : pas surlignée dans le classement.
- **« Provisoire »** : retiré quand tout le monde est arrivé.
- **Alerte** : aucune sur cet écran.
- **iPad portrait** : le cadre Réglages prend toute la largeur en n'affichant que son titre, comme sur Participants et Boîtiers.
- **Non testé** : les deux recherches, les pastilles de catégorie, les classements qui changent pendant la course, la bulle sur iPad et téléphone. Web et iPhone compilent, sans avoir été ouverts.

## 8 — Puces de prêt
- Le prototype n'a que la carte de l'accueil. La page vient de `puces-pret-parc.md` § 10 à 13 et de `GestionPuces.html` de chrono-swift, rangée selon les règles de placement : parcs en colonne, tableau du parc, filtres par état, alerte des puces non rendues, fiche par thèmes.
- Fonctionnent, sur les données de test seulement : choix du parc, filtres, recherche, À vérifier, En service, HS et Perdue (les deux derniers avec confirmation), Remise en service, Rendue, « Voir » depuis l'alerte. Ajouter des puces, Inventaire, Importer, Imprimer, Exporter, Prêter et Pile changée sont visibles, sans effet.
- Générateur : un stock de trois parcs. Les chiffres de l'accueil et de Participants en sont tirés ; les puces prêtées du tableau sont celles de Participants.
- Captures : `08-cards-*` (tirage fixe). Celles des autres écrans sont refaites : le tirage de test a changé, ainsi que les puces prêtées des autres événements.

### Écarts et points ouverts de l'écran 8
- **Nom** : « Puces de prêt » comme le prototype ; la spec dit « Gestion des puces ».
- **Reportés** : cocher plusieurs puces et Déplacer vers, changer une puce de parc, créer ou supprimer un parc, modifier la note ou l'emplacement à la main, tri par les titres, phrases d'explication des actions d'état.
- **Onglet Puces de prêt de la course** (En circulation, Fin de journée, pièce d'identité) : absent des 6 onglets et du prototype.
- **Réglages** : seulement la description du parc, vide sans générateur.
- **Confirmation de HS et Perdue** : dans la fiche, pas dans une fenêtre.
- **« au parc »** : l'accueil et Participants comptent les puces prêtables, la bande du bas compte aussi les HS et les perdues.
- **Puce perdue** : le tableau montre son dernier emplacement sans le dire ; seule la fiche écrit « Dernier lieu connu ». Pas de Rendue tant qu'elle est perdue.
- **Tableau** : des colonnes disparaissent quand la place manque, sans défilement de côté. Le dernier emprunteur n'apparaît alors que dans l'historique de la fiche.
- **Date de pile** : seulement pour les SI-Card 10, 11 et SIAC ; les SI-Card 8 et 9 sont supposées sans pile à changer, à confirmer.
- **Autres événements** : les puces prêtées des événements à venir et passés sont retirées, sauf 1 à 3 non rendues sur des événements passés.
- **Non testé** : recherche et loupe du téléphone, Perdue…, Annuler, Remise en service, contenu de la bulle Réglages, ouverture depuis la carte de l'accueil. Web et iPhone compilent, sans avoir été ouverts.

## 9 — Boîtiers du club
- Le prototype n'a que la carte de l'accueil. La page vient de `lot1b-boitiers-serie.md` § 3 à 6 et de `GestionBoitiersV2.html` de chrono-swift, sur le modèle de la page Puces : encart de lecture et parcs en colonne, tableau du parc, filtres par état, alerte des piles à changer, fiche par thèmes.
- Fonctionnent, sur les données de test seulement : choix du parc, filtres, recherche, Démo › Lire un boîtier, À vérifier, En service, En réparation, Remis en service, Perdu et Détruit (les deux derniers avec confirmation), Pile changée, « Voir » depuis l'alerte. Ajouter des boîtiers, Inventaire, Importer, Imprimer et Exporter sont visibles, sans effet.
- Générateur : un stock de trois parcs, avec les boîtiers de l'événement du jour dans le parc que l'onglet Boîtiers indique. Les chiffres de l'accueil en sont tirés.
- Captures : `09-club-stations-*` (tirage fixe). Celles des autres écrans sont refaites : le tirage de test a changé.

### Écarts et points ouverts de l'écran 9
- **Nom** : « Boîtiers » comme le prototype ; la spec dit « Gestion des boîtiers ». Le titre est le même que l'onglet d'un événement.
- **Reportés** : déroulé de l'inventaire, Afficher la mémoire, Ajouter à une course, Reconfigurer, cocher plusieurs boîtiers et Déplacer vers, changer un boîtier de parc, créer ou supprimer un parc, modifier la note à la main, tri par les titres, choix de l'année dans Pile changée.
- **Encart de lecture** : montre l'heure, la pile et la note, pas la mémoire ni le mode. Pas de boîtier inconnu à la lecture : la démonstration lit seulement les boîtiers du stock. Il garde le dernier boîtier lu, sans limite de 30 s.
- **Réglages** : seulement la description du parc, vide sans générateur.
- **Pile à changer** : Perdu… et Détruit… ajoutés à Pile changée…, pour pouvoir encore déclarer le boîtier.
- **En réparation** : direct, sans date ni lieu demandés.
- **Confirmation de Perdu et Détruit** : dans la fiche, pas dans une fenêtre.
- **Pile changée…** : note l'année 2026 sans rien demander, écrite en dur dans le générateur ; la charge affiche « — » jusqu'à la prochaine lecture.
- **Seuil de pile** : « Pile à changer » sous 3,2 V. La spec fixe ce seuil pour l'orange et laisse à la station le jugement de pile faible.
- **Tableau** : des colonnes disparaissent quand la place manque, sans défilement de côté. Pile et Charge passent alors dans la colonne Note quand elles posent problème ; en iPad portrait, le n° de série passe sous le code et l'heure décalée va dans la Note.
- **Stock** : seuls les boîtiers de l'événement du jour y sont ; ceux des autres événements n'y figurent pas.
- **Accueil** : compte Club et Secours, sans Non identifiés. « piles à changer » remplace « piles faibles » du prototype.
- **Non testé** : recherche et loupe, filtres, changement de parc depuis la colonne, À vérifier, En réparation, Remis en service, clic final sur Marquer perdu ou détruit, Annuler, boîtier perdu relu, contenu de la bulle Réglages, téléphone 320 px, bulle d'alerte sur iPad et téléphone, ouverture depuis la carte de l'accueil. Web et iPhone compilent, sans avoir été ouverts.

## 10 — Coureurs et clubs
- Le prototype n'a que la carte de l'accueil. La page vient de `coureurs-clubs.md` et de `mise-en-page-universelle.md` de chrono-swift, rangée selon les règles de placement : pas de colonne, tableau de la base, filtres par licence, alerte des points à vérifier, fiche par blocs.
- Fonctionnent, sur les données de test seulement : tri par les titres, filtres, recherche, « Voir » depuis l'alerte, Supprimer avec confirmation. Ajouter un coureur, Importer FFCO, Exporter, Fusionner des clubs, Modifier et Chercher le licencié sont visibles, sans effet.
- Générateur : une base de 280 à 340 fiches inventées, avec les participants de l'événement du jour et les emprunteurs de puces. Les chiffres de l'accueil en sont tirés.
- Captures : `10-runner-base-*` (tirage fixe). Celles des autres écrans sont refaites : le tirage de test a changé.

### Écarts et points ouverts de l'écran 10
- **Taille de la base** : quelques centaines de fiches, au lieu d'environ 9 000 en vrai. Tenue de Slint à cette taille à tester si le code sert à l'app.
- **Reportés** : fenêtres de contrôle des doublons et des puces sur deux fiches, fusion, aperçu et lecture des imports FFCO, colonnes facultatives (Sexe, Courses, Dernière course), filtres par club, ligue et catégorie, contact (téléphone, e-mail).
- **Tableau** : pas de défilement de côté, malgré ce que permet la spec sur cet écran ; des colonnes disparaissent quand la place manque. « Découverte » et « Loisir » écrits courts dans le tableau et les filtres ; nom complet dans la fiche.
- **Doublons** : règle simplifiée, une fiche saisie au nom et prénom exacts d'un licencié.
- **Recherche** : ignore les majuscules, pas les accents ; pas la recherche de la saisie d'un coureur.
- **Fiche** : à droite en large, par-dessus sinon, au lieu de la modale de la spec. Modifier… n'est pas en orange, pour garder un seul bouton orange par écran.
- **Téléphone** : pas de découpage aux coins arrondis de la carte, trop haute pour le mode de dessin (carte noire sinon). La première et la dernière ligne peuvent dépasser légèrement.
- **Données inventées** : le sexe et l'année suivent la catégorie, un prénom peut ne pas y correspondre. Le code du club individuel est inventé.
- **Générateur des boîtiers** : parfois beaucoup de piles à changer (10 sur 28 vu sur un tirage), à baisser si besoin.
- **Fiche ouverte seule** : vue une fois sur une capture iPad, sans clic programmé, jamais revue. Cause non établie.
- **Non testé** : recherche et loupe, filtres, tri décroissant, Annuler, contenu de la bulle Réglages, puce partagée et puce non rendue dans la fiche, coureur non relicencié, fiche et bulle sur téléphone et iPad, iPad paysage, téléphone 320 px après la correction, page vide sans données, performances sur un vrai téléphone. Le web compile, sans avoir été ouvert.

### Validations de Robin
- Écran 1 : validé à l'œil le 03/11, sur Mac et iPhone, avant le passage du logo en Instrument Sans.
- Vrai iPhone et Mac : Robin confirme le 03/11 que tout fonctionne partout sur la version corrigée, sauf la flèche de retour sur téléphone (trop petite, pas centrée). Flèche corrigée ensuite. Étape 0 validée par Robin le 03/11.
- iPhone en portrait seulement : accepté.
- Noms du code en anglais : décidé.

### Écarts et points ouverts de l'étape 0
- **Accueil simplifié** : seulement la structure du prototype (Événements, Le club). Carte en fond, bloc Direct, listes À venir et Passés, chiffres des cartes, indicateurs de l'en-tête et nom du club arrivent à l'écran 1.
- **Réglages** : le prototype n'a aucune entrée, donc pas de page pour l'instant. Emplacement à décider.
- **Direct** : 5ᵉ onglet comme le cadrage le demande, alors qu'il est à part dans le prototype. Sans point vivant pour l'instant (écran 6).
- **Retour** : « ‹ Accueil » écrit avec un caractère de la police, comme sur `02-participants.png`, pas une icône ; « ‹ » seul en largeur étroite.
- **iPhone en portrait seulement** : en paysage, Slint ne décale pas le contenu de l'encoche (vu sur le simulateur après rotation). Cause exacte non recherchée.
- **`web/index.html`** : deux couleurs de fond en dur (fond de page clair et sombre), visibles seulement pendant le chargement, avant que Slint dessine.
