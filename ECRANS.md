# Écrans de la maquette

Un écran à la fois, dans cet ordre. On ne passe au suivant qu'après validation de Robin.
Chaque écran : 3 dispositions (étroite, moyenne, large), tactile et souris, clair et sombre.

| # | Écran | Statut |
|---|---|---|
| 0 | Mise en place (dépôt, projet, publication web) | à valider par Robin |
| 1 | Accueil | à faire |
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

### Validations de Robin
- Vrai iPhone et Mac : Robin confirme le 03/11 que tout fonctionne partout sur la version corrigée, sauf la flèche de retour sur téléphone (trop petite, pas centrée). Flèche corrigée ensuite, à revoir par Robin.
- iPhone en portrait seulement : accepté.
- Noms du code en anglais : décidé.

### Écarts et points ouverts de l'étape 0
- **Accueil simplifié** : seulement la structure du prototype (Événements, Le club). Carte en fond, bloc Direct, listes À venir et Passés, chiffres des cartes, indicateurs de l'en-tête et nom du club arrivent à l'écran 1.
- **Réglages** : le prototype n'a aucune entrée, donc pas de page pour l'instant. Emplacement à décider.
- **Direct** : 5ᵉ onglet comme le cadrage le demande, alors qu'il est à part dans le prototype. Sans point vivant pour l'instant (écran 6).
- **Retour** : « ‹ Accueil » écrit avec un caractère de la police, comme sur `02-participants.png`, pas une icône ; « ‹ » seul en largeur étroite.
- **iPhone en portrait seulement** : en paysage, Slint ne décale pas le contenu de l'encoche (vu sur le simulateur après rotation). Cause exacte non recherchée.
- **`web/index.html`** : deux couleurs de fond en dur (fond de page clair et sombre), visibles seulement pendant le chargement, avant que Slint dessine.
