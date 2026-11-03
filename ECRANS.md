# Écrans de la maquette

Un écran à la fois, dans cet ordre. On ne passe au suivant qu'après validation de Robin.
Chaque écran : 3 dispositions (étroite, moyenne, large), tactile et souris, clair et sombre.

| # | Écran | Statut |
|---|---|---|
| 0 | Mise en place (dépôt, projet, publication web) | validé par Robin ; publication web à tester au premier push |
| 1 | Accueil | validé par Robin |
| 2 | Course › Circuits | à faire |
| 3 | Course › Participants | à faire |
| 4 | Course › Départs | à faire |
| 5 | Course › Boîtiers | à faire |
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
