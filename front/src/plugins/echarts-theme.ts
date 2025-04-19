/**
 * plugins/echarts-theme.ts
 *
 * Configuration du thème echarts basé sur les couleurs Vuetify
 */

import * as echarts from 'echarts';
import vuetify from './vuetify';
import type { App } from 'vue';
import { computed } from 'vue';
import { THEME_KEY } from 'vue-echarts';

// Fonction pour créer les thèmes echarts à partir des thèmes Vuetify
export function setupEchartsTheme() {
  // On récupère les thèmes de Vuetify (light et dark)
  const vuetifyThemes = vuetify.theme.themes.value;

  // Thème light
  const lightColors = vuetifyThemes.light.colors;
  echarts.registerTheme('vuetify-light', {
    color: [
      lightColors.primary,
      lightColors.secondary,
      lightColors.info,
      lightColors.success,
      lightColors.warning,
      lightColors.error,
      lightColors['primary-darken-1'],
      lightColors['secondary-darken-1'],
    ],
    textStyle: {
      color: lightColors['on-background'],
    },
    title: {
      textStyle: {
        color: lightColors['on-background'],
      },
      subtextStyle: {
        color: lightColors['on-surface-variant'],
      },
    },
    line: {
      itemStyle: {
        borderWidth: 1,
      },
      lineStyle: {
        width: 2,
      },
      symbolSize: 4,
      symbol: 'circle',
      smooth: false,
    },
    radar: {
      itemStyle: {
        borderWidth: 1,
      },
      lineStyle: {
        width: 2,
      },
      symbolSize: 4,
      symbol: 'circle',
      smooth: false,
    },
    bar: {
      itemStyle: {
        barBorderWidth: 0,
        barBorderColor: lightColors['surface-variant'],
      },
    },
    pie: {
      itemStyle: {
        borderWidth: 0,
        borderColor: lightColors['surface-variant'],
      },
    },
    scatter: {
      itemStyle: {
        borderWidth: 0,
        borderColor: lightColors['surface-variant'],
      },
    },
    boxplot: {
      itemStyle: {
        borderWidth: 0,
        borderColor: lightColors['surface-variant'],
      },
    },
    parallel: {
      itemStyle: {
        borderWidth: 0,
        borderColor: lightColors['surface-variant'],
      },
    },
    sankey: {
      itemStyle: {
        borderWidth: 0,
        borderColor: lightColors['surface-variant'],
      },
    },
    funnel: {
      itemStyle: {
        borderWidth: 0,
        borderColor: lightColors['surface-variant'],
      },
    },
    gauge: {
      itemStyle: {
        borderWidth: 0,
        borderColor: lightColors['surface-variant'],
      },
    },
    candlestick: {
      itemStyle: {
        color: lightColors.error,
        color0: lightColors.success,
        borderColor: lightColors.error,
        borderColor0: lightColors.success,
        borderWidth: 1,
      },
    },
    graph: {
      itemStyle: {
        borderWidth: 0,
        borderColor: lightColors['surface-variant'],
      },
      lineStyle: {
        width: 1,
        color: lightColors['on-surface-variant'],
      },
      symbolSize: 4,
      symbol: 'circle',
      smooth: false,
      color: [
        lightColors.primary,
        lightColors.secondary,
        lightColors.info,
        lightColors.success,
        lightColors.warning,
        lightColors.error,
      ],
      label: {
        color: lightColors['on-background'],
      },
    },
    map: {
      itemStyle: {
        areaColor: lightColors.surface,
        borderColor: lightColors['surface-variant'],
        borderWidth: 0.5,
      },
      label: {
        color: lightColors['on-background'],
      },
      emphasis: {
        itemStyle: {
          areaColor: lightColors['surface-light'],
          borderColor: lightColors.primary,
          borderWidth: 1,
        },
        label: {
          color: lightColors['on-primary'],
        },
      },
    },
    visualMap: {
      inRange: {
        color: [
          '#f5eee6',
          '#e3d5c1',
          '#d1bba0',
          '#bb9b7d',
          '#a67c5b',
          '#8e6042',
          '#72452d',
          '#5a3322',
          '#412316',
          '#2a140b',
        ],
      },
    },
    geo: {
      itemStyle: {
        areaColor: lightColors.surface,
        borderColor: lightColors['surface-variant'],
        borderWidth: 0.5,
      },
      label: {
        color: lightColors['on-background'],
      },
      emphasis: {
        itemStyle: {
          areaColor: lightColors['surface-light'],
          borderColor: lightColors.primary,
          borderWidth: 1,
        },
        label: {
          color: lightColors['on-primary'],
        },
      },
    },
    categoryAxis: {
      axisLine: {
        show: true,
        lineStyle: {
          color: lightColors['surface-variant'],
        },
      },
      axisTick: {
        show: true,
        lineStyle: {
          color: lightColors['surface-variant'],
        },
      },
      axisLabel: {
        show: true,
        color: lightColors['on-surface-variant'],
      },
      splitLine: {
        show: false,
        lineStyle: {
          color: [lightColors['surface-light']],
        },
      },
      splitArea: {
        show: false,
        areaStyle: {
          color: [lightColors['surface'], lightColors['surface-light']],
        },
      },
    },
    valueAxis: {
      axisLine: {
        show: true,
        lineStyle: {
          color: lightColors['surface-variant'],
        },
      },
      axisTick: {
        show: true,
        lineStyle: {
          color: lightColors['surface-variant'],
        },
      },
      axisLabel: {
        show: true,
        color: lightColors['on-surface-variant'],
      },
      splitLine: {
        show: true,
        lineStyle: {
          color: [lightColors['surface-light']],
        },
      },
      splitArea: {
        show: false,
        areaStyle: {
          color: [lightColors['surface'], lightColors['surface-light']],
        },
      },
    },
    logAxis: {
      axisLine: {
        show: true,
        lineStyle: {
          color: lightColors['surface-variant'],
        },
      },
      axisTick: {
        show: true,
        lineStyle: {
          color: lightColors['surface-variant'],
        },
      },
      axisLabel: {
        show: true,
        color: lightColors['on-surface-variant'],
      },
      splitLine: {
        show: true,
        lineStyle: {
          color: [lightColors['surface-light']],
        },
      },
      splitArea: {
        show: false,
        areaStyle: {
          color: [lightColors['surface'], lightColors['surface-light']],
        },
      },
    },
    timeAxis: {
      axisLine: {
        show: true,
        lineStyle: {
          color: lightColors['surface-variant'],
        },
      },
      axisTick: {
        show: true,
        lineStyle: {
          color: lightColors['surface-variant'],
        },
      },
      axisLabel: {
        show: true,
        color: lightColors['on-surface-variant'],
      },
      splitLine: {
        show: true,
        lineStyle: {
          color: [lightColors['surface-light']],
        },
      },
      splitArea: {
        show: false,
        areaStyle: {
          color: [lightColors['surface'], lightColors['surface-light']],
        },
      },
    },
    toolbox: {
      iconStyle: {
        borderColor: lightColors['on-surface-variant'],
      },
      emphasis: {
        iconStyle: {
          borderColor: lightColors['on-background'],
        },
      },
    },
    legend: {
      textStyle: {
        color: lightColors['on-background'],
      },
    },
    tooltip: {
      axisPointer: {
        lineStyle: {
          color: lightColors['surface-variant'],
          width: 1,
        },
        crossStyle: {
          color: lightColors['surface-variant'],
          width: 1,
        },
      },
      backgroundColor: lightColors['surface-bright'],
      borderColor: lightColors['surface-variant'],
      borderWidth: 1,
      textStyle: {
        color: lightColors['on-surface'],
      },
    },
    timeline: {
      lineStyle: {
        color: lightColors['on-surface-variant'],
        width: 1,
      },
      itemStyle: {
        borderWidth: 1,
        color: lightColors.primary,
      },
      controlStyle: {
        color: lightColors.primary,
        borderColor: lightColors['surface-variant'],
        borderWidth: 0.5,
      },
      checkpointStyle: {
        color: lightColors.primary,
        borderColor: lightColors['primary-darken-1'],
      },
      label: {
        color: lightColors['on-background'],
      },
      emphasis: {
        itemStyle: {
          color: lightColors['primary-darken-1'],
        },
        controlStyle: {
          color: lightColors.primary,
          borderColor: lightColors['surface-variant'],
          borderWidth: 0.5,
        },
        label: {
          color: lightColors['on-background'],
        },
      },
    },

    dataZoom: {
      backgroundColor: lightColors.surface,
      dataBackgroundColor: lightColors['surface-light'],
      fillerColor: 'rgba(197, 156, 108, 0.2)',
      handleColor: lightColors.primary,
      handleSize: '100%',
      textStyle: {
        color: lightColors['on-background'],
      },
    },
  });

  // Thème dark
  const darkColors = vuetifyThemes.dark.colors;
  echarts.registerTheme('vuetify-dark', {
    color: [
      darkColors.primary,
      darkColors.secondary,
      darkColors.info,
      darkColors.success,
      darkColors.warning,
      darkColors.error,
      darkColors['primary-darken-1'],
      darkColors['secondary-darken-1'],
    ],
    textStyle: {
      color: darkColors['on-background'],
    },
    title: {
      textStyle: {
        color: darkColors['on-background'],
      },
      subtextStyle: {
        color: darkColors['on-surface-variant'],
      },
    },
    line: {
      itemStyle: {
        borderWidth: 1,
      },
      lineStyle: {
        width: 2,
      },
      symbolSize: 4,
      symbol: 'circle',
      smooth: false,
    },
    radar: {
      itemStyle: {
        borderWidth: 1,
      },
      lineStyle: {
        width: 2,
      },
      symbolSize: 4,
      symbol: 'circle',
      smooth: false,
    },
    bar: {
      itemStyle: {
        barBorderWidth: 0,
        barBorderColor: darkColors['surface-variant'],
      },
    },
    pie: {
      itemStyle: {
        borderWidth: 0,
        borderColor: darkColors['surface-variant'],
      },
    },
    scatter: {
      itemStyle: {
        borderWidth: 0,
        borderColor: darkColors['surface-variant'],
      },
    },
    boxplot: {
      itemStyle: {
        borderWidth: 0,
        borderColor: darkColors['surface-variant'],
      },
    },
    parallel: {
      itemStyle: {
        borderWidth: 0,
        borderColor: darkColors['surface-variant'],
      },
    },
    sankey: {
      itemStyle: {
        borderWidth: 0,
        borderColor: darkColors['surface-variant'],
      },
    },
    funnel: {
      itemStyle: {
        borderWidth: 0,
        borderColor: darkColors['surface-variant'],
      },
    },
    gauge: {
      itemStyle: {
        borderWidth: 0,
        borderColor: darkColors['surface-variant'],
      },
    },
    candlestick: {
      itemStyle: {
        color: darkColors.error,
        color0: darkColors.success,
        borderColor: darkColors.error,
        borderColor0: darkColors.success,
        borderWidth: 1,
      },
    },
    graph: {
      itemStyle: {
        borderWidth: 0,
        borderColor: darkColors['surface-variant'],
      },
      lineStyle: {
        width: 1,
        color: darkColors['on-surface-variant'],
      },
      symbolSize: 4,
      symbol: 'circle',
      smooth: false,
      color: [
        darkColors.primary,
        darkColors.secondary,
        darkColors.info,
        darkColors.success,
        darkColors.warning,
        darkColors.error,
      ],
      label: {
        color: darkColors['on-background'],
      },
    },
    map: {
      itemStyle: {
        areaColor: darkColors.surface,
        borderColor: darkColors['surface-variant'],
        borderWidth: 0.5,
      },
      label: {
        color: darkColors['on-background'],
      },
      emphasis: {
        itemStyle: {
          areaColor: darkColors['surface-light'],
          borderColor: darkColors.primary,
          borderWidth: 1,
        },
        label: {
          color: darkColors['on-primary'],
        },
      },
    },
    geo: {
      itemStyle: {
        areaColor: darkColors.surface,
        borderColor: darkColors['surface-variant'],
        borderWidth: 0.5,
      },
      label: {
        color: darkColors['on-background'],
      },
      emphasis: {
        itemStyle: {
          areaColor: darkColors['surface-light'],
          borderColor: darkColors.primary,
          borderWidth: 1,
        },
        label: {
          color: darkColors['on-primary'],
        },
      },
    },
    categoryAxis: {
      axisLine: {
        show: true,
        lineStyle: {
          color: darkColors['surface-variant'],
        },
      },
      axisTick: {
        show: true,
        lineStyle: {
          color: darkColors['surface-variant'],
        },
      },
      axisLabel: {
        show: true,
        color: darkColors['on-surface-variant'],
      },
      splitLine: {
        show: false,
        lineStyle: {
          color: [darkColors['surface-light']],
        },
      },
      splitArea: {
        show: false,
        areaStyle: {
          color: [darkColors['surface'], darkColors['surface-light']],
        },
      },
    },
    valueAxis: {
      axisLine: {
        show: true,
        lineStyle: {
          color: darkColors['surface-variant'],
        },
      },
      axisTick: {
        show: true,
        lineStyle: {
          color: darkColors['surface-variant'],
        },
      },
      axisLabel: {
        show: true,
        color: darkColors['on-surface-variant'],
      },
      splitLine: {
        show: true,
        lineStyle: {
          color: [darkColors['surface-light']],
        },
      },
      splitArea: {
        show: false,
        areaStyle: {
          color: [darkColors['surface'], darkColors['surface-light']],
        },
      },
    },
    logAxis: {
      axisLine: {
        show: true,
        lineStyle: {
          color: darkColors['surface-variant'],
        },
      },
      axisTick: {
        show: true,
        lineStyle: {
          color: darkColors['surface-variant'],
        },
      },
      axisLabel: {
        show: true,
        color: darkColors['on-surface-variant'],
      },
      splitLine: {
        show: true,
        lineStyle: {
          color: [darkColors['surface-light']],
        },
      },
      splitArea: {
        show: false,
        areaStyle: {
          color: [darkColors['surface'], darkColors['surface-light']],
        },
      },
    },
    timeAxis: {
      axisLine: {
        show: true,
        lineStyle: {
          color: darkColors['surface-variant'],
        },
      },
      axisTick: {
        show: true,
        lineStyle: {
          color: darkColors['surface-variant'],
        },
      },
      axisLabel: {
        show: true,
        color: darkColors['on-surface-variant'],
      },
      splitLine: {
        show: true,
        lineStyle: {
          color: [darkColors['surface-light']],
        },
      },
      splitArea: {
        show: false,
        areaStyle: {
          color: [darkColors['surface'], darkColors['surface-light']],
        },
      },
    },
    toolbox: {
      iconStyle: {
        borderColor: darkColors['on-surface-variant'],
      },
      emphasis: {
        iconStyle: {
          borderColor: darkColors['on-background'],
        },
      },
    },
    legend: {
      textStyle: {
        color: darkColors['on-background'],
      },
    },
    tooltip: {
      axisPointer: {
        lineStyle: {
          color: darkColors['surface-variant'],
          width: 1,
        },
        crossStyle: {
          color: darkColors['surface-variant'],
          width: 1,
        },
      },
      backgroundColor: darkColors['surface-bright'],
      borderColor: darkColors['surface-variant'],
      borderWidth: 1,
      textStyle: {
        color: darkColors['on-surface'],
      },
    },
    timeline: {
      lineStyle: {
        color: darkColors['on-surface-variant'],
        width: 1,
      },
      itemStyle: {
        borderWidth: 1,
        color: darkColors.primary,
      },
      controlStyle: {
        color: darkColors.primary,
        borderColor: darkColors['surface-variant'],
        borderWidth: 0.5,
      },
      checkpointStyle: {
        color: darkColors.primary,
        borderColor: darkColors['primary-darken-1'],
      },
      label: {
        color: darkColors['on-background'],
      },
      emphasis: {
        itemStyle: {
          color: darkColors['primary-darken-1'],
        },
        controlStyle: {
          color: darkColors.primary,
          borderColor: darkColors['surface-variant'],
          borderWidth: 0.5,
        },
        label: {
          color: darkColors['on-background'],
        },
      },
    },
    visualMap: {
      inRange: {
        color: [
          '#f5eee6',
          '#e3d5c1',
          '#d1bba0',
          '#bb9b7d',
          '#a67c5b',
          '#8e6042',
          '#72452d',
          '#5a3322',
          '#412316',
          '#2a140b',
        ],
      },
    },
    dataZoom: {
      backgroundColor: darkColors.surface,
      dataBackgroundColor: darkColors['surface-light'],
      fillerColor: 'rgba(164, 122, 86, 0.2)',
      handleColor: darkColors.primary,
      handleSize: '100%',
      textStyle: {
        color: darkColors['on-background'],
      },
    },
  });
}

// Fonction pour configurer le thème ECharts globalement dans l'application Vue
export function setupGlobalEchartsTheme(app: App) {
  // Surveiller le thème actuel de Vuetify
  const currentTheme = computed(() => {
    return vuetify.theme.global.current.value.dark ? 'vuetify-dark' : 'vuetify-light';
  });

  // Fournir le thème ECharts globalement
  app.provide(THEME_KEY, currentTheme);
}

export default setupEchartsTheme;
