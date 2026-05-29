(function () {
  'use strict';

  var asciiHeroArt = [
    '██╗   ██╗ ██████╗ ██╗██████╗ ',
    '██║   ██║██╔═══██╗██║██╔══██╗',
    '██║   ██║██║   ██║██║██║  ██║',
    '╚██╗ ██╔╝██║   ██║██║██║  ██║',
    ' ╚████╔╝ ╚██████╔╝██║██████╔╝',
    '  ╚═══╝   ╚═════╝ ╚═╝╚═════╝ '
  ].join('\n');

  var voidshellArt = [
    '██╗   ██╗ ██████╗ ██╗██████╗ ',
    '██║   ██║██╔═══██╗██║██╔══██╗',
    '██║   ██║██║   ██║██║██║  ██║',
    '╚██╗ ██╔╝██║   ██║██║██║  ██║',
    ' ╚████╔╝ ╚██████╔╝██║██████╔╝',
    '  ╚═══╝   ╚═════╝ ╚═╝╚═════╝ '
  ].join('\n');

  var heroEl = document.getElementById('ascii-hero');
  var heroTitle = document.getElementById('hero-title');
  var heroTagline = document.getElementById('hero-tagline');
  var heroDesc = document.getElementById('hero-desc');

  function typeElement(el, text, speed, callback) {
    el.textContent = '';
    el.classList.add('typing');
    var i = 0;
    function step() {
      if (i < text.length) {
        el.textContent += text.charAt(i);
        i++;
        setTimeout(step, speed);
      } else if (callback) {
        callback();
      }
    }
    step();
  }

  function typeAsciiArt(el, art, speed, callback) {
    el.textContent = '';
    var fullText = art;
    var i = 0;
    function step() {
      if (i < fullText.length) {
        el.textContent += fullText.charAt(i);
        i++;
        setTimeout(step, speed);
      } else {
        if (callback) callback();
      }
    }
    el.classList.add('visible');
    step();
  }

  function initHero() {
    typeAsciiArt(heroEl, voidshellArt, 8, function () {
      typeElement(heroTitle, 'CARAPACE', 70, function () {
        typeElement(heroTagline, 'Immortality has a price. Your humanity is the currency.', 35, function () {
          typeElement(heroDesc, 'A procedurally-generated terminal RPG set in a biological grimdark world. A century after civilization collapsed, immortal crustaceans rule the deep trenches — their blood-thirsty hybrids walk among humans. Harvest the telomerase enzyme from the dead. Trade in blood. Survive the harvest.', 15);
        });
      });
    });
  }

  function initScrollReveal() {
    var reveals = document.querySelectorAll('.reveal');
    if (!reveals.length) return;

    var observer = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (entry.isIntersecting) {
          entry.target.classList.add('visible');
          observer.unobserve(entry.target);
        }
      });
    }, {
      threshold: 0.15,
      rootMargin: '0px 0px -40px 0px'
    });

    reveals.forEach(function (el) {
      observer.observe(el);
    });
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', function () {
      initHero();
      initScrollReveal();
    });
  } else {
    initHero();
    initScrollReveal();
  }
})();
