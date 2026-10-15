(ns midnight-alley.world-test
  "The STM demo: many rats hit the same food scrap / roof at the same instant."
  (:require [clojure.test :refer [deftest is testing]]
            [midnight-alley.world :as w])
  (:import (java.util.concurrent CountDownLatch)))

(defn- race
  "Run `f` on n threads released at the same moment; count how many returned true."
  [n f]
  (let [start (CountDownLatch. 1)
        futs (doall (repeatedly n #(future (.await start) (f))))]
    (.countDown start)
    (count (filter true? (map deref futs)))))

(deftest safe-mode-one-rat-gets-the-food
  (let [world (w/new-world {:safe? true})
        id (w/drop-food! world 10.0)]
    (is (= 1 (race 100 #(w/claim-food! world id))))
    (is (= 0 (w/food-count world)))))

(deftest safe-mode-one-rat-gets-the-roof
  (let [world (w/new-world {:safe? true})
        winners (atom 0)]
    (is (= 1 (race 100 #(w/claim-roof! world 0 (swap! winners inc)))))
    (is (some? (w/roof-owner world 0)))))

(deftest broken-mode-shows-ghost-food
  ;; Probabilistic by nature: the check-then-act has a 1ms window, so with 100
  ;; threads released together more than one almost certainly gets through.
  (testing "two rats eat the same scrap and the count goes negative"
    (let [world (w/new-world {:safe? false})
          id (w/drop-food! world 10.0)
          winners (race 100 #(w/claim-food! world id))]
      (is (> winners 1))
      (is (neg? (w/food-count world))))))

(deftest broken-mode-shows-two-rats-on-one-roof
  (let [world (w/new-world {:safe? false})
        winners (atom 0)]
    (is (> (race 100 #(w/claim-roof! world 0 (swap! winners inc))) 1))))
