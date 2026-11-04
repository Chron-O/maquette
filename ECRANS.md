# Écrans de la maquette

Un écran à la fois, dans cet ordre. On ne passe au suivant qu'après validation de Robin.
Chaque écran : 3 dispositions (étroite, moyenne, large), tactile et souris, clair et sombre.

| # | Écran | Statut |
|---|---|---|
| 0 | Mise en place (dépôt, projet, publication web) | validé par Robin ; publication web à tester au premier push |
| 1 | Accueil | validé par Robin |
| 2 | Course › Circuits | validé par Robin ; Mixte à maquetter, modification des circuits à faire plus tard |
| 3 | Course › Participants | validé par Robin ; nouveau participant et attribution de puce à faire plus tard |
| 4 | Course › Départs | validé par Robin ; réglage des groupes, vrai tirage et modifications à faire plus tard |
| 5 | Course › Boîtiers | validé par Robin ; Programmer, Choisir un boîtier, Corriger les temps et boîtiers spéciaux à faire plus tard |
| 6 | Course › Direct | à faire |
| 7 | Course › Résultats | à faire |
| 8 | Puces (stock de prêt) | à faire |
| 9 | Boîtiers (inventaire du club) | à faire |
| 10 | Coureurs et clubs (base fédérale) | à faire |

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
- **Fiche, avant et après** : « D » et « A » écrits pour Départ et Arrivée, au lieu de leurs symboles.
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
