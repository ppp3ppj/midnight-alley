(ns midnight-alley.world
  "The shared state of the alley and the only safe ways to change it.

  Safe mode (default): food scraps and roofs are refs, and eating a scrap or
  claiming a roof is a dosync transaction, so two rats can never win the same one.

  Broken mode: the same API over plain atoms and a mutable array, using an
  unprotected check-then-act. Two rats can both 'win' the same scrap or roof,
  the food count goes negative, and two rats sleep on one roof.")

(def world-width 1000.0)
(def night-len 50.0)   ; seconds of darkness
(def morning-len 12.0) ; seconds of light, rats go hide and sleep
(def max-rats 60)
(def initial-rats 30)

(def roof-layout
  [{:x 120.0 :tier 1} {:x 380.0 :tier 2} {:x 620.0 :tier 1} {:x 880.0 :tier 2}])

(defn new-world
  ([] (new-world {}))
  ([{:keys [safe?] :or {safe? true}}]
   {:safe?      (atom safe?)
    :next-id    (atom 0)
    ;; safe store (STM)
    :food       (ref {})                                    ; id -> x
    :roofs      (vec (for [r roof-layout] (ref (assoc r :owner nil))))
    ;; broken store (no protection on purpose)
    :b-food     (atom {})
    :b-roofs    (object-array (count roof-layout))          ; owner id or nil
    :food-count (atom 0)                                    ; broken mode only; drifts negative
    ;; the colony diary
    :diary      (atom {:born 0 :eaten 0 :night 1})
    :clock      (atom {:t 0.0 :morning? false})
    :rats       (atom [])}))                                ; vector of rat agents

(defn safe? [world] @(:safe? world))

(defn next-id! [world] (swap! (:next-id world) inc))

;; ---------------------------------------------------------------- food

(defn drop-food! [world x]
  (let [id (next-id! world)]
    (if (safe? world)
      (dosync (alter (:food world) assoc id x))
      (do (swap! (:b-food world) assoc id x)
          (swap! (:food-count world) inc)))
    id))

(defn food-snapshot
  "Map of scrap id -> x position."
  [world]
  (if (safe? world) @(:food world) @(:b-food world)))

(defn food-count [world]
  (if (safe? world) (count @(:food world)) @(:food-count world)))

(defn claim-food!
  "Try to eat scrap `id`. Returns true only for the rat that wins it."
  [world id]
  (if (safe? world)
    (let [food (:food world)]
      (dosync
        (when (contains? @food id)
          (alter food dissoc id)
          true)))
    (when (contains? @(:b-food world) id)
      (Thread/sleep 1)                                      ; widen the race window for the demo
      (swap! (:b-food world) dissoc id)
      (swap! (:food-count world) dec)
      true)))

(defn note-eaten! [world]
  (swap! (:diary world) update :eaten inc))

;; ---------------------------------------------------------------- roofs

(defn roof-count [_world] (count roof-layout))

(defn roof-x [_world i] (:x (nth roof-layout i)))

(defn roof-owner [world i]
  (if (safe? world)
    (:owner @(nth (:roofs world) i))
    (let [^objects arr (:b-roofs world)] (aget arr i))))

(defn claim-roof!
  "Try to take roof `i` for rat `rat-id`. Returns true only for the winner."
  [world i rat-id]
  (if (safe? world)
    (let [roof (nth (:roofs world) i)]
      (dosync
        (when (nil? (:owner @roof))
          (alter roof assoc :owner rat-id)
          true)))
    (let [^objects arr (:b-roofs world)]
      (when (nil? (aget arr i))
        (Thread/sleep 1)                                    ; widen the race window for the demo
        (aset arr i rat-id)
        true))))

(defn release-roof! [world i rat-id]
  (if (safe? world)
    (let [roof (nth (:roofs world) i)]
      (dosync
        (when (= rat-id (:owner @roof))
          (alter roof assoc :owner nil))))
    (let [^objects arr (:b-roofs world)]
      (aset arr i nil))))

(defn roofs-snapshot [world]
  (vec (for [i (range (roof-count world))]
         (assoc (nth roof-layout i) :owner (roof-owner world i)))))

;; ---------------------------------------------------------------- reset

(defn reset-stores!
  "Empty both food stores and free every roof (used when switching modes)."
  [world]
  (dosync
    (ref-set (:food world) {})
    (doseq [r (:roofs world)] (alter r assoc :owner nil)))
  (reset! (:b-food world) {})
  (reset! (:food-count world) 0)
  (let [^objects arr (:b-roofs world)]
    (dotimes [i (alength arr)] (aset arr i nil))))
